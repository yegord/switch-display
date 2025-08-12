#[forbid(unsafe_code)]
mod screen;
mod screen_controller;
mod switch;

fn main() {
    let controller = screen_controller::ScreenController::Xrandr;

    let screen = controller.get_outputs();
    println!("screen = {:?}", screen);

    let switch_plan = switch::build_switch_plan(&screen);
    println!("switch_plan = {:?}", switch_plan);

    println!("outputs_to_disable = {:?}", switch_plan.outputs_to_disable.iter().map(|output| output.name.as_str()).collect::<Vec<_>>());
    println!("outputs_to_enable = {:?}", switch_plan.outputs_to_enable.iter().map(|output| output.name.as_str()).collect::<Vec<_>>());

    let best_resolution = switch::choose_best_resolution(&switch_plan.outputs_to_enable, None);
    println!("best_resolution = {:?}", best_resolution);

    controller.switch_outputs(&switch_plan, best_resolution)
}
