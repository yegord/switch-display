#[forbid(unsafe_code)]

use clap::Parser;

mod screen;
mod screen_controller;
mod switch;

#[derive(Parser)]
#[command(author, version, about, arg_required_else_help(true))]
struct Args {
    /// Method to use for querying and setting output resolutions
    #[arg(long)]
    controller: screen_controller::ScreenController,
    /// When choosing a resolution, choose one with at least this refresh rate.
    /// The value is specified in thousands of Hz.
    #[arg(long)]
    min_refresh_rate: Option<i32>,
}

fn main() {
    // TODO: logging
    let args = Args::parse();

    let screen = args.controller.get_outputs();
    println!("screen = {screen:?}");

    let switch_plan = switch::build_switch_plan(&screen);
    println!("switch_plan = {switch_plan:?}");

    println!(
        "outputs_to_disable = {:?}",
        switch_plan
            .outputs_to_disable
            .iter()
            .map(|output| output.name.as_str())
            .collect::<Vec<_>>()
    );
    println!(
        "outputs_to_enable = {:?}",
        switch_plan
            .outputs_to_enable
            .iter()
            .map(|output| output.name.as_str())
            .collect::<Vec<_>>()
    );

    let best_resolution = switch::choose_best_resolution(&switch_plan.outputs_to_enable, args.min_refresh_rate);
    println!("best_resolution = {best_resolution:?}");

    args.controller.switch_outputs(&switch_plan, best_resolution)
}
