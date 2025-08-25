use crate::screen;
use crate::switch::SwitchPlan;
use std::collections::HashMap;
use std::iter::Iterator;
use x11rb::CURRENT_TIME;
use x11rb::connection::Connection;
use x11rb::protocol::randr::GetOutputInfoReply;
use x11rb::protocol::{randr, randr::ConnectionExt};
use x11rb::rust_connection::RustConnection;

pub(super) fn get_outputs() -> screen::Screen {
    let (conn, screen_num) =
        RustConnection::connect(None).expect("unable to connect to X11 display");
    let screen = &conn.setup().roots[screen_num];

    let screen_resources = conn
        .randr_get_screen_resources(screen.root)
        .expect("randr_get_screen_resources call failed")
        .reply()
        .expect("randr_get_screen_resources returned an error");

    log::trace!("screen_resources = {screen_resources:?}");

    let modes: HashMap<_, _> = screen_resources
        .modes
        .iter()
        .map(|mode| (mode.id, mode))
        .collect();

    let outputs = screen_resources
        .outputs
        .iter()
        .map(|&output_id| {
            conn.randr_get_output_info(output_id, screen_resources.config_timestamp)
                .expect("randr_get_output_info call failed")
                .reply()
                .expect("randr_get_output_info returned an error")
        })
        .inspect(|output| log::trace!("output = {output:?}"))
        .map(|output| randr_output_into_output(output, &modes))
        .collect();

    screen::Screen { outputs }
}

pub(super) fn switch_outputs(switch_plan: &SwitchPlan, resolution: Option<screen::Resolution>) {
    let (conn, screen_num) =
        RustConnection::connect(None).expect("unable to connect to X11 display");
    let screen = &conn.setup().roots[screen_num];

    let screen_resources = conn
        .randr_get_screen_resources(screen.root)
        .expect("randr_get_screen_resources call failed")
        .reply()
        .expect("randr_get_screen_resources returned an error");

    let modes: HashMap<_, _> = screen_resources
        .modes
        .iter()
        .map(|mode| (mode.id, mode))
        .collect();

    let outputs: HashMap<_, _> = screen_resources
        .outputs
        .iter()
        .copied()
        .map(|output_id| {
            (
                output_id,
                conn.randr_get_output_info(output_id, screen_resources.config_timestamp)
                    .expect("randr_get_output_info call failed")
                    .reply()
                    .expect("randr_get_output_info returned an error"),
            )
        })
        .collect();

    let crtcs: HashMap<_, _> = screen_resources
        .crtcs
        .iter()
        .copied()
        .map(|crtc_id| {
            (
                crtc_id,
                conn.randr_get_crtc_info(crtc_id, screen_resources.config_timestamp)
                    .expect("randr_get_crtc_info call failed")
                    .reply()
                    .expect("randr_get_crtc_info returned an error"),
            )
        })
        .collect();

    let crtc_configs = compute_crtc_configs(switch_plan, resolution, &modes, &outputs, &crtcs);

    for (&crtc_id, crtc_config) in &crtc_configs {
        log::trace!("crtc_id = {crtc_id} crtc_config = {crtc_config:?}");
        conn.randr_set_crtc_config(
            crtc_id,
            CURRENT_TIME,
            screen_resources.config_timestamp,
            crtc_config.x,
            crtc_config.y,
            crtc_config.mode.map(|mode| mode.id).unwrap_or(0),
            crtc_config.rotation,
            &crtc_config.outputs,
        )
        .expect("randr_set_crtc_config call failed")
        .reply()
        .expect("randr_set_crtc_config returned an error");
    }

    if let Some(screen_size) = compute_screen_size(&crtc_configs, &outputs) {
        log::trace!("screen_size = {screen_size:?}");
        conn.randr_set_screen_size(
            screen.root,
            screen_size.width,
            screen_size.height,
            screen_size.mm_width,
            screen_size.mm_height,
        )
        .expect("randr_set_screen_size call failed")
        .check()
        .expect("randr_set_screen_size returned an error");
    }
}

fn randr_output_into_output(
    output: randr::GetOutputInfoReply,
    modes: &HashMap<randr::Mode, &randr::ModeInfo>,
) -> screen::Output {
    let name =
        String::from_utf8(output.name).expect("output name should normally be a valid UTF-8");
    let connected = output.connection == randr::Connection::CONNECTED;
    let enabled = output.crtc != 0;
    let location = screen::Location::from_output_name(&name);

    let modes = mode_ids_to_modes(&output.modes, modes)
        .filter(|mode| is_admissible(mode))
        .map(randr_mode_to_mode)
        .collect();

    screen::Output {
        name,
        connected,
        enabled,
        modes,
        location,
    }
}

fn mode_ids_to_modes<'a>(
    mode_ids: &[randr::Mode],
    modes: &HashMap<randr::Mode, &'a randr::ModeInfo>,
) -> impl Iterator<Item = &'a randr::ModeInfo> {
    mode_ids
        .iter()
        .map(|mode_id| modes.get(mode_id).copied().expect("invalid mode id"))
}

fn is_admissible(mode: &randr::ModeInfo) -> bool {
    !mode.mode_flags.contains(randr::ModeFlag::DOUBLE_SCAN)
}

fn randr_mode_to_mode(mode: &randr::ModeInfo) -> screen::Mode {
    screen::Mode {
        resolution: randr_mode_to_resolution(mode),
        refresh_rate: compute_refresh_rate(mode),
    }
}

fn randr_mode_to_resolution(mode: &randr::ModeInfo) -> screen::Resolution {
    screen::Resolution {
        width: mode.width as u32,
        height: mode.height as u32,
    }
}

fn compute_refresh_rate(mode: &randr::ModeInfo) -> u32 {
    if mode.htotal > 0 && mode.vtotal > 0 {
        u32::try_from(mode.dot_clock as u64 * 1000 / (mode.htotal as u64 * mode.vtotal as u64))
            .expect("refresh rate should fit into u32")
    } else {
        0
    }
}

#[derive(Debug)]
struct CrtcConfig<'a> {
    x: i16,
    y: i16,
    mode: Option<&'a randr::ModeInfo>,
    rotation: randr::Rotation,
    outputs: Vec<randr::Output>,
}

impl<'a> CrtcConfig<'a> {
    fn from_crtc(
        crtc: &randr::GetCrtcInfoReply,
        modes: &HashMap<randr::Mode, &'a randr::ModeInfo>,
    ) -> Self {
        CrtcConfig {
            x: crtc.x,
            y: crtc.y,
            mode: if crtc.mode == 0 {
                None
            } else {
                Some(modes.get(&crtc.mode).expect("bad mode id"))
            },
            rotation: crtc.rotation,
            outputs: crtc.outputs.clone(),
        }
    }
}

fn compute_crtc_configs<'a>(
    switch_plan: &SwitchPlan,
    resolution: Option<screen::Resolution>,
    modes: &HashMap<u32, &'a randr::ModeInfo>,
    outputs: &HashMap<randr::Output, randr::GetOutputInfoReply>,
    crtcs: &HashMap<randr::Crtc, randr::GetCrtcInfoReply>,
) -> HashMap<randr::Crtc, CrtcConfig<'a>> {
    let outputs_to_disable = outputs.iter().filter(|(_, output)| {
        switch_plan
            .outputs_to_disable
            .iter()
            .any(|output_to_disable| output_to_disable.name.as_bytes() == output.name)
    });

    let outputs_to_enable = outputs.iter().filter(|(_, output)| {
        switch_plan
            .outputs_to_enable
            .iter()
            .any(|output_to_enable| output_to_enable.name.as_bytes() == output.name)
    });

    let mut crtc_configs: HashMap<_, _> = crtcs
        .iter()
        .map(|(crtc_id, crtc)| (*crtc_id, CrtcConfig::from_crtc(crtc, modes)))
        .collect();

    for (output_id, output) in outputs_to_disable {
        assert!(output.crtc != 0);
        let crtc_config = crtc_configs.get_mut(&output.crtc).expect("invalid crtc id");
        assert!(crtc_config.outputs.contains(output_id));
        crtc_config.outputs.retain(|id| id != output_id);
        if crtc_config.outputs.is_empty() {
            crtc_config.mode = None;
        }
    }

    for (output_id, output) in outputs_to_enable {
        let crtc_config = if output.crtc != 0 {
            let crtc_config = crtc_configs.get_mut(&output.crtc).expect("invalid crtc id");
            assert!(crtc_config.outputs.contains(output_id));
            crtc_config
        } else {
            let crtc_id = output
                .crtcs
                .iter()
                .copied()
                .find(|crtc_id| {
                    crtc_configs
                        .get(crtc_id)
                        .is_some_and(|crtc_config| crtc_config.outputs.is_empty())
                })
                .expect("no free crtcs available for output");

            let crtc_config = crtc_configs.get_mut(&crtc_id).expect("invalid crtc id");
            assert!(!crtc_config.outputs.contains(output_id));
            crtc_config.outputs.push(*output_id);
            crtc_config
        };

        crtc_config.x = 0;
        crtc_config.y = 0;
        crtc_config.mode =
            Some(choose_best_mode(output, modes, resolution).expect("output has no modes"));
        crtc_config.rotation = randr::Rotation::ROTATE0;
    }

    crtc_configs
}

fn choose_best_mode<'a>(
    output: &randr::GetOutputInfoReply,
    modes: &HashMap<randr::Mode, &'a randr::ModeInfo>,
    resolution: Option<screen::Resolution>,
) -> Option<&'a randr::ModeInfo> {
    struct Candidate<'a> {
        preferred: bool,
        mode: &'a randr::ModeInfo,
    }

    let candidates: Vec<_> = mode_ids_to_modes(&output.modes, modes)
        .enumerate()
        .map(|(i, mode)| Candidate {
            preferred: i < output.num_preferred as usize,
            mode,
        })
        .collect();

    if let Some(resolution) = resolution {
        if let Some(candidate) = candidates
            .iter()
            .filter(|candidate| candidate.preferred || is_admissible(candidate.mode))
            .filter(|candidate| randr_mode_to_resolution(candidate.mode) == resolution)
            .max_by_key(|candidate| (candidate.preferred, compute_refresh_rate(candidate.mode)))
        {
            return Some(candidate.mode);
        }
    }

    candidates
        .iter()
        .max_by_key(|candidate| {
            (
                candidate.preferred,
                randr_mode_to_resolution(candidate.mode).area(),
                compute_refresh_rate(candidate.mode),
            )
        })
        .map(|candidate| candidate.mode)
}

#[derive(Debug)]
struct ScreenSize {
    width: u16,
    height: u16,
    mm_width: u32,
    mm_height: u32,
}

fn compute_screen_size(
    crtc_configs: &HashMap<randr::Crtc, CrtcConfig>,
    outputs: &HashMap<randr::Output, GetOutputInfoReply>,
) -> Option<ScreenSize> {
    let bboxes: Vec<_> = crtc_configs
        .values()
        .filter_map(|crtc_config| {
            crtc_config.mode.map(|mode| {
                (
                    crtc_config.x as i32,
                    crtc_config.y as i32,
                    crtc_config.x as i32 + mode.width as i32,
                    crtc_config.y as i32 + mode.height as i32,
                )
            })
        })
        .collect();

    let min_x = bboxes.iter().map(|bbox| bbox.0).min();
    let min_y = bboxes.iter().map(|bbox| bbox.1).min();
    let max_x = bboxes.iter().map(|bbox| bbox.2).max();
    let max_y = bboxes.iter().map(|bbox| bbox.3).max();

    let outputs: Vec<_> = crtc_configs
        .values()
        .flat_map(|crtc_config| crtc_config.outputs.iter())
        .map(|output_id| outputs.get(output_id).expect("invalid output id"))
        .collect();

    let mm_width = outputs
        .iter()
        .map(|output| output.mm_width)
        .filter(|&w| w > 0)
        .max();
    let mm_height = outputs
        .iter()
        .map(|output| output.mm_height)
        .filter(|&h| h > 0)
        .max();

    if let (Some(min_x), Some(min_y), Some(max_x), Some(max_y)) = (min_x, min_y, max_x, max_y) {
        let width = u16::try_from(max_x - min_x).expect("too large screen width");
        let height = u16::try_from(max_y - min_y).expect("too large screen height");
        let mm_width = mm_width.unwrap_or_else(|| px_to_mm(width));
        let mm_height = mm_height.unwrap_or_else(|| px_to_mm(height));

        Some(ScreenSize {
            width,
            height,
            mm_width,
            mm_height,
        })
    } else {
        None
    }
}

fn px_to_mm(px: u16) -> u32 {
    const DPI: f32 = 96.0;
    const MM_PER_INCH: f32 = 25.4;

    ((px as f32 / DPI) * MM_PER_INCH).round() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "needs X11, manual"]
    fn get_outputs_smoke_test() {
        let screen = get_outputs();
        println!("{:?}", screen);
        assert!(!screen.outputs.is_empty());
        for output in &screen.outputs {
            assert!(!output.connected || !output.modes.is_empty());
        }
    }
}
