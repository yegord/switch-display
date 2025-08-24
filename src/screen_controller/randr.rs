use crate::screen;
use x11rb::connection::Connection;
use x11rb::protocol::randr::{Connection as RandrConnection, *};
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

    let outputs = screen_resources
        .outputs
        .iter()
        .map(|&output_id| {
            let output = conn
                .randr_get_output_info(output_id, screen_resources.config_timestamp)
                .expect("randr_get_output_info call failed")
                .reply()
                .expect("randr_get_output_info returned an error");

            let name = String::from_utf8(output.name)
                .expect("output name should normally be a valid UTF-8");
            let connected = output.connection == RandrConnection::CONNECTED;
            let enabled = output.crtc != 0;
            let location = screen::Location::from_output_name(&name);

            let modes = output
                .modes
                .iter()
                .map(|&mode_id| {
                    screen_resources
                        .modes
                        .iter()
                        .find(|m| m.id == mode_id)
                        .expect("unable to find mode info by mode id")
                })
                .filter(|mode| !mode.mode_flags.contains(ModeFlag::DOUBLE_SCAN))
                .map(|mode| {
                    let resolution = screen::Resolution {
                        width: mode.width as u32,
                        height: mode.height as u32,
                    };

                    let refresh_rate = if mode.htotal > 0 && mode.vtotal > 0 {
                        u32::try_from(
                            mode.dot_clock as u64 * 1000
                                / (mode.htotal as u64 * mode.vtotal as u64),
                        )
                        .expect("refresh rate should fit into u32")
                    } else {
                        0
                    };

                    screen::Mode {
                        resolution,
                        refresh_rate,
                    }
                })
                .collect();

            screen::Output {
                name,
                connected,
                enabled,
                modes,
                location,
            }
        })
        .collect();

    screen::Screen { outputs }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "needs X11, manual"]
    fn get_outputs_smoke_test() {
        println!("{:?}", get_outputs());
    }
}
