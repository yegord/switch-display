#![forbid(unsafe_code)]
mod screen;
mod screen_controller;
mod switch;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, arg_required_else_help(true))]
struct Args {
    /// Method to use for querying and setting output resolutions.
    #[arg(long, env = "SWITCH_DISPLAY_CONTROLLER")]
    controller: screen_controller::ScreenController,
    /// When choosing a resolution, choose one with at least this refresh rate.
    /// The value is specified in millihertz, i.e. 60000 is 60 Hz.
    #[arg(long, env = "SWITCH_DISPLAY_MIN_REFRESH_RATE")]
    min_refresh_rate: Option<u32>,
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    let screen = args.controller.get_outputs();
    log::trace!("screen = {screen:?}");

    let switch_plan = switch::build_switch_plan(&screen);
    log::trace!("switch_plan = {switch_plan:?}");

    log::debug!(
        "outputs_to_disable = {:?}",
        switch_plan
            .outputs_to_disable
            .iter()
            .map(|output| output.name.as_str())
            .collect::<Vec<_>>()
    );
    log::debug!(
        "outputs_to_enable = {:?}",
        switch_plan
            .outputs_to_enable
            .iter()
            .map(|output| output.name.as_str())
            .collect::<Vec<_>>()
    );

    let best_resolution =
        switch::choose_best_resolution(&switch_plan.outputs_to_enable, args.min_refresh_rate);
    log::debug!("best_resolution = {best_resolution:?}");

    args.controller
        .switch_outputs(&switch_plan, best_resolution)
}
