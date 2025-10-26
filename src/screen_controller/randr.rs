use crate::screen;
use crate::switch::SwitchPlan;
use std::collections::HashMap;
use std::iter::Iterator;
use x11rb::CURRENT_TIME;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::Timestamp;
use x11rb::protocol::{randr, randr::ConnectionExt};
use x11rb::rust_connection::RustConnection;

pub(super) struct RandrClient {
    conn: RustConnection,
    screen_num: usize,
    config_timestamp: Timestamp,
    modes: HashMap<randr::Mode, randr::ModeInfo>,
    outputs: HashMap<randr::Output, randr::GetOutputInfoReply>,
    crtcs: HashMap<randr::Crtc, randr::GetCrtcInfoReply>,
}

impl RandrClient {
    pub(super) fn new() -> Self {
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
            .into_iter()
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
            .inspect(|(output_id, output)| log::trace!("outputs[{output_id}] = {output:?}"))
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

        Self {
            conn,
            screen_num,
            config_timestamp: screen_resources.config_timestamp,
            modes,
            outputs,
            crtcs,
        }
    }

    pub(super) fn get_outputs(&self) -> screen::Screen {
        let outputs = self
            .outputs
            .values()
            .map(|output| randr_output_to_output(output, &self.modes))
            .collect();

        screen::Screen { outputs }
    }

    pub(super) fn switch_outputs(
        &mut self,
        switch_plan: &SwitchPlan,
        resolution: Option<screen::Resolution>,
    ) {
        update_crtcs(
            switch_plan,
            resolution,
            &self.modes,
            &mut self.outputs,
            &mut self.crtcs,
        );

        let screen = &self.conn.setup().roots[self.screen_num];

        for (&crtc_id, crtc_config) in &self.crtcs {
            log::trace!("crtc_id = {crtc_id} crtc_config = {crtc_config:?}");
            self.conn
                .randr_set_crtc_config(
                    crtc_id,
                    CURRENT_TIME,
                    self.config_timestamp,
                    crtc_config.x,
                    crtc_config.y,
                    crtc_config.mode,
                    crtc_config.rotation,
                    &crtc_config.outputs,
                )
                .expect("randr_set_crtc_config call failed")
                .reply()
                .expect("randr_set_crtc_config returned an error");
        }

        if let Some(screen_size) = compute_screen_size(&self.modes, &self.outputs, &self.crtcs) {
            log::trace!("screen_size = {screen_size:?}");
            self.conn
                .randr_set_screen_size(
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
}

fn randr_output_to_output(
    output: &randr::GetOutputInfoReply,
    modes: &HashMap<randr::Mode, randr::ModeInfo>,
) -> screen::Output {
    let name = String::from_utf8(output.name.clone())
        .expect("output name should normally be a valid UTF-8");
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
    modes: &'a HashMap<randr::Mode, randr::ModeInfo>,
) -> impl Iterator<Item = &'a randr::ModeInfo> {
    mode_ids.iter().map(|mode_id| {
        let mode = modes.get(mode_id).expect("invalid mode id");
        assert_eq!(*mode_id, mode.id);
        mode
    })
}

fn is_admissible(mode: &randr::ModeInfo) -> bool {
    !mode.mode_flags.contains(randr::ModeFlag::DOUBLE_SCAN)
}

fn randr_mode_to_mode(mode: &randr::ModeInfo) -> screen::Mode {
    screen::Mode {
        resolution: randr_mode_to_resolution(mode),
        refresh_rate_millihz: compute_refresh_rate_millihz(mode),
    }
}

fn randr_mode_to_resolution(mode: &randr::ModeInfo) -> screen::Resolution {
    screen::Resolution {
        width: mode.width as u32,
        height: mode.height as u32,
    }
}

fn compute_refresh_rate_millihz(mode: &randr::ModeInfo) -> u32 {
    if mode.htotal > 0 && mode.vtotal > 0 {
        u32::try_from(mode.dot_clock as u64 * 1000 / (mode.htotal as u64 * mode.vtotal as u64))
            .expect("refresh rate should fit into u32")
    } else {
        0
    }
}

fn update_crtcs(
    switch_plan: &SwitchPlan,
    resolution: Option<screen::Resolution>,
    modes: &HashMap<u32, randr::ModeInfo>,
    outputs: &mut HashMap<randr::Output, randr::GetOutputInfoReply>,
    crtcs: &mut HashMap<randr::Crtc, randr::GetCrtcInfoReply>,
) {
    let outputs_to_disable = outputs
        .iter_mut()
        .filter(|(_, output)| output.crtc != 0)
        .filter(|(_, output)| {
            switch_plan
                .outputs_to_disable
                .iter()
                .any(|output_to_disable| output_to_disable.name.as_bytes() == output.name)
        });

    for (output_id, output) in outputs_to_disable {
        assert!(output.crtc != 0);
        let crtc = crtcs.get_mut(&output.crtc).expect("invalid crtc id");
        assert!(crtc.outputs.contains(output_id));
        crtc.outputs.retain(|id| id != output_id);
        if crtc.outputs.is_empty() {
            crtc.mode = 0;
        }
        output.crtc = 0;
    }

    let outputs_to_enable = outputs.iter_mut().filter(|(_, output)| {
        switch_plan
            .outputs_to_enable
            .iter()
            .any(|output_to_enable| output_to_enable.name.as_bytes() == output.name)
    });

    for (output_id, output) in outputs_to_enable {
        let crtc = if output.crtc != 0 {
            let crtc = crtcs.get_mut(&output.crtc).expect("invalid crtc id");
            assert!(crtc.outputs.contains(output_id));
            crtc
        } else {
            let crtc_id = output
                .crtcs
                .iter()
                .copied()
                .find(|crtc_id| {
                    crtcs
                        .get(crtc_id)
                        .expect("invalid crtc id")
                        .outputs
                        .is_empty()
                })
                .expect("no free crtcs available for output");

            let crtc = crtcs.get_mut(&crtc_id).expect("invalid crtc id");
            assert!(!crtc.outputs.contains(output_id));
            crtc.outputs.push(*output_id);
            output.crtc = crtc_id;
            crtc
        };

        crtc.x = 0;
        crtc.y = 0;
        crtc.mode = choose_best_mode(output, modes, resolution).expect("output has no modes");
        crtc.rotation = randr::Rotation::ROTATE0;
    }

    assert!(crtcs.iter().all(
        |(&crtc_id, crtc)| (crtc.mode == 0) == crtc.outputs.is_empty()
            && crtc.outputs.iter().all(|output_id| {
                outputs
                    .get(output_id)
                    .is_some_and(|output| output.crtc == crtc_id)
            })
    ));
    assert!(
        outputs
            .iter()
            .filter(|(_, output)| output.crtc != 0)
            .all(|(output_id, output)| crtcs
                .get(&output.crtc)
                .is_some_and(|crtc| crtc.outputs.contains(output_id)))
    );
}

fn choose_best_mode(
    output: &randr::GetOutputInfoReply,
    modes: &HashMap<randr::Mode, randr::ModeInfo>,
    resolution: Option<screen::Resolution>,
) -> Option<randr::Mode> {
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
        .filter(|candidate| candidate.preferred || is_admissible(candidate.mode))
        .collect();

    if let Some(resolution) = resolution
        && let Some(candidate) = candidates
            .iter()
            .filter(|candidate| randr_mode_to_resolution(candidate.mode) == resolution)
            .max_by_key(|candidate| {
                (
                    candidate.preferred,
                    compute_refresh_rate_millihz(candidate.mode),
                )
            })
    {
        return Some(candidate.mode.id);
    }

    candidates
        .iter()
        .max_by_key(|candidate| {
            (
                candidate.preferred,
                randr_mode_to_resolution(candidate.mode).area(),
                compute_refresh_rate_millihz(candidate.mode),
            )
        })
        .map(|candidate| candidate.mode.id)
}

#[derive(Debug, PartialEq, Eq)]
struct ScreenSize {
    width: u16,
    height: u16,
    mm_width: u32,
    mm_height: u32,
}

fn compute_screen_size(
    modes: &HashMap<randr::Mode, randr::ModeInfo>,
    outputs: &HashMap<randr::Output, randr::GetOutputInfoReply>,
    crtcs: &HashMap<randr::Crtc, randr::GetCrtcInfoReply>,
) -> Option<ScreenSize> {
    let bboxes: Vec<_> = crtcs
        .values()
        .filter(|crtc| crtc.mode != 0)
        .map(|crtc| {
            let mode = modes.get(&crtc.mode).expect("invalid mode id");
            (
                crtc.x as i32,
                crtc.y as i32,
                crtc.x as i32 + mode.width as i32,
                crtc.y as i32 + mode.height as i32,
            )
        })
        .collect();

    let min_x = bboxes.iter().map(|bbox| bbox.0).min();
    let min_y = bboxes.iter().map(|bbox| bbox.1).min();
    let max_x = bboxes.iter().map(|bbox| bbox.2).max();
    let max_y = bboxes.iter().map(|bbox| bbox.3).max();

    if let (Some(min_x), Some(min_y), Some(max_x), Some(max_y)) = (min_x, min_y, max_x, max_y) {
        let width = u16::try_from(max_x - min_x).expect("too large screen width");
        let height = u16::try_from(max_y - min_y).expect("too large screen height");

        let (mm_width, mm_height) = crtcs
            .values()
            .flat_map(|crtc_config| crtc_config.outputs.iter())
            .map(|output_id| outputs.get(output_id).expect("invalid output id"))
            .map(|output| (output.mm_width, output.mm_height))
            .filter(|(w, h)| *w != 0 && *h != 0)
            .max_by_key(|(w, h)| *w as u64 * *h as u64)
            .unwrap_or_else(|| (px_to_mm(width), px_to_mm(height)));

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

    (px as f32 * (MM_PER_INCH / DPI)).round() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    use maplit::hashmap;

    #[test]
    #[ignore = "needs X11, manual"]
    fn get_outputs_smoke_test() {
        // Arrange
        let client = RandrClient::new();

        // Act
        let screen = client.get_outputs();
        log::trace!("screen = {screen:?}");

        // Assert
        assert!(!screen.outputs.is_empty());
        for output in &screen.outputs {
            assert!(!output.connected || !output.modes.is_empty());
        }
    }

    #[test]
    #[ignore = "needs X11, manual"]
    fn switch_outputs_smoke_test() {
        // Arrange
        let mut client = RandrClient::new();
        let switch_plan = SwitchPlan {
            outputs_to_disable: Vec::new(),
            outputs_to_enable: Vec::new(),
        };

        // Act
        let screen = client.get_outputs();
        client.switch_outputs(&switch_plan, None);
        let new_screen = client.get_outputs();

        // Assert
        assert_eq!(screen, new_screen);
    }

    #[test]
    fn test_randr_output_to_output_on_internal_connected_enabled_output() {
        // Arrange
        let randr_output = randr::GetOutputInfoReply {
            crtc: 42,
            connection: randr::Connection::CONNECTED,
            modes: vec![1, 2],
            name: b"eDP-1".to_vec(),
            ..Default::default()
        };

        let modes = hashmap! {
            1 => randr::ModeInfo {
                id: 1,
                width: 1920,
                height: 1080,
                dot_clock: 138700000,
                htotal: 2080,
                vtotal: 1111,
                ..Default::default()
            },
            2 => randr::ModeInfo {
                id: 2,
                width: 3840,
                height: 2160,
                dot_clock: 138700000,
                htotal: 2080,
                vtotal: 1111,
                mode_flags: randr::ModeFlag::DOUBLE_SCAN,
                ..Default::default()
            },
        };

        // Act
        let output = randr_output_to_output(&randr_output, &modes);

        // Assert
        assert_eq!(
            output,
            screen::Output {
                name: "eDP-1".to_owned(),
                enabled: true,
                connected: true,
                modes: vec! {screen::Mode {
                    resolution: screen::Resolution {
                        width: 1920,
                        height: 1080,
                    },
                    refresh_rate_millihz: 60020,
                }},
                location: screen::Location::Internal,
            }
        );
    }

    #[test]
    fn test_randr_output_to_output_on_external_disconnected_output() {
        // Arrange
        let randr_output = randr::GetOutputInfoReply {
            connection: randr::Connection::DISCONNECTED,
            name: b"HDMI-1".to_vec(),
            ..Default::default()
        };

        let modes = HashMap::new();

        // Act
        let output = randr_output_to_output(&randr_output, &modes);

        // Assert
        assert_eq!(
            output,
            screen::Output {
                name: "HDMI-1".to_owned(),
                enabled: false,
                connected: false,
                modes: Vec::new(),
                location: screen::Location::External,
            }
        );
    }

    #[test]
    fn test_is_admissible() {
        assert!(is_admissible(&randr::ModeInfo {
            ..Default::default()
        }));
        assert!(!is_admissible(&randr::ModeInfo {
            mode_flags: randr::ModeFlag::DOUBLE_SCAN,
            ..Default::default()
        }));
    }

    #[test]
    fn test_randr_mode_to_mode() {
        assert_eq!(
            randr_mode_to_mode(&randr::ModeInfo {
                width: 1920,
                height: 1080,
                dot_clock: 138700000,
                htotal: 2080,
                vtotal: 1111,
                ..Default::default()
            }),
            screen::Mode {
                resolution: screen::Resolution {
                    width: 1920,
                    height: 1080,
                },
                refresh_rate_millihz: 60020,
            }
        );
    }

    #[test]
    fn test_randr_mode_to_resolution() {
        assert_eq!(
            randr_mode_to_resolution(&randr::ModeInfo {
                width: 640,
                height: 480,
                ..Default::default()
            }),
            screen::Resolution {
                width: 640,
                height: 480
            }
        );
    }

    #[test]
    fn test_compute_refresh_rate_millihz() {
        assert_eq!(
            compute_refresh_rate_millihz(&randr::ModeInfo {
                dot_clock: 138700000,
                htotal: 2080,
                vtotal: 1111,
                ..Default::default()
            }),
            60020
        );
        assert_eq!(
            compute_refresh_rate_millihz(&randr::ModeInfo {
                dot_clock: 138700000,
                htotal: 0,
                vtotal: 1111,
                ..Default::default()
            }),
            0
        );
        assert_eq!(
            compute_refresh_rate_millihz(&randr::ModeInfo {
                dot_clock: 138700000,
                htotal: 2080,
                vtotal: 0,
                ..Default::default()
            }),
            0
        );
    }

    #[test]
    fn test_update_crtcs() {
        // Arrange
        let modes = hashmap! {
            1 => randr::ModeInfo {
                id: 1,
                width: 1920,
                height: 1080,
                dot_clock: 138700000,
                htotal: 2080,
                vtotal: 1111,
                ..Default::default()
            }
        };

        let mut randr_outputs = hashmap! {
            10 => randr::GetOutputInfoReply {
                crtc: 20,
                connection: randr::Connection::CONNECTED,
                crtcs: vec![20, 21, 22],
                modes: vec![1],
                name: b"eDP-1".to_vec(),
                ..Default::default()
            },
            11 => randr::GetOutputInfoReply {
                crtc: 0,
                connection: randr::Connection::DISCONNECTED,
                crtcs: vec![20, 21, 22],
                modes: vec![1],
                name: b"HDMI-1".to_vec(),
                ..Default::default()
            },
            12 => randr::GetOutputInfoReply {
                crtc: 0,
                connection: randr::Connection::CONNECTED,
                crtcs: vec![20, 21, 22],
                modes: vec![1],
                name: b"HDMI-2".to_vec(),
                ..Default::default()
            },
            13 => randr::GetOutputInfoReply {
                crtc: 21,
                connection: randr::Connection::CONNECTED,
                crtcs: vec![20, 21, 22],
                modes: vec![1],
                name: b"HDMI-3".to_vec(),
                ..Default::default()
            },
            14 => randr::GetOutputInfoReply {
                crtc: 22,
                connection: randr::Connection::CONNECTED,
                crtcs: vec![20, 21, 22],
                modes: vec![1],
                name: b"HDMI-4".to_vec(),
                ..Default::default()
            },
        };

        let mut crtcs = hashmap! {
            20 => randr::GetCrtcInfoReply {
                mode: 1,
                outputs: vec![10],
                ..Default::default()
            },
            21 => randr::GetCrtcInfoReply {
                x: 10,
                y: 20,
                mode: 1,
                rotation: randr::Rotation::ROTATE90,
                outputs: vec![13],
                ..Default::default()
            },
            22 => randr::GetCrtcInfoReply {
                mode: 1,
                outputs: vec![14],
                ..Default::default()
            },
        };

        let resolution = None;

        let outputs: Vec<_> = [10, 11, 12, 13]
            .iter()
            .map(|output_id| randr_output_to_output(randr_outputs.get(output_id).unwrap(), &modes))
            .collect();

        let switch_plan = SwitchPlan {
            outputs_to_disable: vec![&outputs[0], &outputs[1]],
            outputs_to_enable: vec![&outputs[2], &outputs[3]],
        };

        // Act
        update_crtcs(
            &switch_plan,
            resolution,
            &modes,
            &mut randr_outputs,
            &mut crtcs,
        );

        // Assert
        assert_eq!(randr_outputs.get(&10).unwrap().crtc, 0);
        assert_eq!(randr_outputs.get(&11).unwrap().crtc, 0);
        assert_eq!(randr_outputs.get(&12).unwrap().crtc, 20);
        assert_eq!(randr_outputs.get(&13).unwrap().crtc, 21);
        assert_eq!(randr_outputs.get(&14).unwrap().crtc, 22);

        let crtc1 = crtcs.get(&20).unwrap();
        assert_eq!(crtc1.outputs.as_slice(), [12]);

        let crtc2 = crtcs.get(&21).unwrap();
        assert_eq!(crtc2.outputs.as_slice(), [13]);
        assert_eq!(crtc2.x, 0);
        assert_eq!(crtc2.y, 0);
        assert_eq!(crtc2.mode, 1);
        assert_eq!(crtc2.rotation, randr::Rotation::ROTATE0);
    }

    #[test]
    fn when_no_modes_available_choose_best_mode_returns_none() {
        // Arrange
        let output = randr::GetOutputInfoReply {
            ..Default::default()
        };
        let modes = HashMap::new();
        let resolution = None;

        // Act
        let mode_id = choose_best_mode(&output, &modes, resolution);

        // Assert
        assert!(mode_id.is_none());
    }

    #[test]
    fn when_no_preferred_or_admissible_mode_available_choose_best_mode_returns_none() {
        // Arrange
        let output = randr::GetOutputInfoReply {
            modes: vec![1],
            ..Default::default()
        };
        let modes = hashmap!(
            1 => randr::ModeInfo{id: 1, mode_flags: randr::ModeFlag::DOUBLE_SCAN, ..Default::default()},
        );
        let resolution = None;

        // Act
        let mode_id = choose_best_mode(&output, &modes, resolution);

        // Assert
        assert!(mode_id.is_none());
    }

    #[test]
    fn when_preferred_but_not_admissible_mode_available_choose_best_mode_returns_it() {
        // Arrange
        let output = randr::GetOutputInfoReply {
            modes: vec![1],
            num_preferred: 1,
            ..Default::default()
        };
        let modes = hashmap!(
            1 => randr::ModeInfo{id: 1, mode_flags: randr::ModeFlag::DOUBLE_SCAN, ..Default::default()},
        );
        let resolution = None;

        // Act
        let mode_id = choose_best_mode(&output, &modes, resolution);

        // Assert
        assert_eq!(mode_id, Some(1));
    }

    #[test]
    fn when_not_preferred_but_admissible_mode_available_choose_best_mode_returns_it() {
        // Arrange
        let output = randr::GetOutputInfoReply {
            modes: vec![1],
            ..Default::default()
        };
        let modes = hashmap!(
            1 => randr::ModeInfo{id: 1, ..Default::default()},
        );
        let resolution = None;

        // Act
        let mode_id = choose_best_mode(&output, &modes, resolution);

        // Assert
        assert_eq!(mode_id, Some(1));
    }

    #[test]
    fn choose_best_mode_prefers_preferred_mode() {
        // Arrange
        let output = randr::GetOutputInfoReply {
            modes: vec![1, 2],
            num_preferred: 1,
            ..Default::default()
        };
        let modes = hashmap!(
            1 => randr::ModeInfo{id: 1, width: 640, height: 480, ..Default::default()},
            2 => randr::ModeInfo{id: 2, width: 800, height: 600, ..Default::default()},
        );
        let resolution = None;

        // Act
        let mode_id = choose_best_mode(&output, &modes, resolution);

        // Assert
        assert_eq!(mode_id, Some(1));
    }

    #[test]
    fn choose_best_mode_prefers_larger_mode() {
        // Arrange
        let output = randr::GetOutputInfoReply {
            modes: vec![1, 2],
            ..Default::default()
        };
        let modes = hashmap!(
            1 => randr::ModeInfo{id: 1, width: 640, height: 480, ..Default::default()},
            2 => randr::ModeInfo{id: 2, width: 800, height: 600, ..Default::default()},
        );
        let resolution = None;

        // Act
        let mode_id = choose_best_mode(&output, &modes, resolution);

        // Assert
        assert_eq!(mode_id, Some(2));
    }

    #[test]
    fn choose_best_mode_prefers_mode_with_higher_refresh_rate() {
        // Arrange
        let output = randr::GetOutputInfoReply {
            modes: vec![1, 2],
            ..Default::default()
        };
        let modes = hashmap!(
            1 => randr::ModeInfo{id: 1, width: 640, height: 480, dot_clock: 1, htotal: 1, vtotal: 1, ..Default::default()},
            2 => randr::ModeInfo{id: 2, width: 640, height: 480, dot_clock: 2, htotal: 1, vtotal: 1, ..Default::default()},
        );
        let resolution = None;

        // Act
        let mode_id = choose_best_mode(&output, &modes, resolution);

        // Assert
        assert_eq!(mode_id, Some(2));
    }

    #[test]
    fn when_resolution_provided_choose_best_mode_prefers_preferred_mode() {
        // Arrange
        let output = randr::GetOutputInfoReply {
            modes: vec![1, 2, 3],
            num_preferred: 1,
            ..Default::default()
        };
        let modes = hashmap!(
            1 => randr::ModeInfo{id: 1, width: 640, height: 480, dot_clock: 1, htotal: 1, vtotal: 1, ..Default::default()},
            2 => randr::ModeInfo{id: 2, width: 640, height: 480, dot_clock: 2, htotal: 1, vtotal: 1, ..Default::default()},
            3 => randr::ModeInfo{id: 3, width: 800, height: 600, dot_clock: 3, htotal: 1, vtotal: 1, ..Default::default()},
        );
        let resolution = Some(screen::Resolution {
            width: 640,
            height: 480,
        });

        // Act
        let mode_id = choose_best_mode(&output, &modes, resolution);

        // Assert
        assert_eq!(mode_id, Some(1));
    }

    #[test]
    fn when_resolution_provided_choose_best_mode_prefers_mode_with_highest_refresh_rate() {
        // Arrange
        let output = randr::GetOutputInfoReply {
            modes: vec![1, 2],
            ..Default::default()
        };
        let modes = hashmap!(
            1 => randr::ModeInfo{id: 1, width: 640, height: 480, dot_clock: 1, htotal: 1, vtotal: 1, ..Default::default()},
            2 => randr::ModeInfo{id: 2, width: 640, height: 480, dot_clock: 2, htotal: 1, vtotal: 1, ..Default::default()},
        );
        let resolution = Some(screen::Resolution {
            width: 640,
            height: 480,
        });

        // Act
        let mode_id = choose_best_mode(&output, &modes, resolution);

        // Assert
        assert_eq!(mode_id, Some(2));
    }

    #[test]
    fn when_no_crtcs_compute_screen_size_returns_none() {
        // Arrange
        let modes = HashMap::new();
        let outputs = HashMap::new();
        let crtcs = HashMap::new();

        // Act
        let size = compute_screen_size(&modes, &crtcs, &outputs);

        // Assert
        assert!(size.is_none());
    }

    #[test]
    fn when_no_crtcs_enabled_compute_screen_size_returns_none() {
        // Arrange
        let modes = HashMap::new();
        let outputs = hashmap! {
            20 => randr::GetOutputInfoReply { ..Default::default() }
        };
        let crtcs = hashmap! {
            10 => randr::GetCrtcInfoReply { mode: 0, outputs: vec!{20}, ..Default::default() }
        };

        // Act
        let size = compute_screen_size(&modes, &outputs, &crtcs);

        // Assert
        assert!(size.is_none());
    }

    #[test]
    fn when_crtcs_enabled_compute_screen_size_returns_bbox_size_and_estimated_mm_size() {
        // Arrange
        let modes = hashmap! {
            1 => randr::ModeInfo {
                width: 640,
                height: 480,
                ..Default::default()
            }
        };
        let outputs = hashmap! {
            10 => randr::GetOutputInfoReply { ..Default::default() },
            11 => randr::GetOutputInfoReply { mm_width: 0, mm_height: 1, ..Default::default() },
        };
        let crtcs = hashmap! {
            20 => randr::GetCrtcInfoReply { x: 0, y: 0, mode: 1, outputs: vec!{10}, ..Default::default() },
            21 => randr::GetCrtcInfoReply { x: -10, y: 10, mode: 1, outputs: vec!{11}, ..Default::default() },
        };

        // Act
        let size = compute_screen_size(&modes, &outputs, &crtcs);

        // Assert
        assert_eq!(
            size,
            Some(ScreenSize {
                width: 650,
                height: 490,
                mm_width: px_to_mm(650),
                mm_height: px_to_mm(490)
            })
        );
    }

    #[test]
    fn when_crtcs_enabled_and_mm_sizes_known_compute_screen_size_returns_bbox_size_and_max_mm_size()
    {
        // Arrange
        let modes = hashmap! {
            1 => randr::ModeInfo {
                width: 640,
                height: 480,
                ..Default::default()
            }
        };
        let outputs = hashmap! {
            10 => randr::GetOutputInfoReply { mm_width: 400, mm_height: 100, ..Default::default() },
            11 => randr::GetOutputInfoReply { mm_width: 220, mm_height: 220, ..Default::default() },
        };
        let crtcs = hashmap! {
            20 => randr::GetCrtcInfoReply { x: 0, y: 0, mode: 1, outputs: vec!{10}, ..Default::default() },
            21 => randr::GetCrtcInfoReply { x: 10, y: -10, mode: 1, outputs: vec!{11}, ..Default::default() },
        };

        // Act
        let size = compute_screen_size(&modes, &outputs, &crtcs);

        // Assert
        assert_eq!(
            size,
            Some(ScreenSize {
                width: 650,
                height: 490,
                mm_width: 220,
                mm_height: 220,
            })
        );
    }

    #[test]
    fn px_to_mm_test() {
        assert_eq!(px_to_mm(0), 0);
        assert_eq!(px_to_mm(u16::MAX), 17339);
    }
}
