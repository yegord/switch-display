#[forbid(unsafe_code)]
mod screen;
mod switch;
mod xrandr;

fn main() {
    let screen = xrandr::query_xrandr();
    println!("screen = {:?}", screen);

    let mut switch_plan = switch::build_switch_plan(&screen);
    println!("switch_plan = {:?}", switch_plan);

    println!("outputs_to_disable = {:?}", switch_plan.outputs_to_disable.iter().map(|output| output.name.as_str()).collect::<Vec<_>>());
    println!("outputs_to_enable = {:?}", switch_plan.outputs_to_enable.iter().map(|output| output.name.as_str()).collect::<Vec<_>>());

    let best_resolution = switch::choose_best_resolution(&switch_plan.outputs_to_enable, None);
    println!("best_resolution = {:?}", best_resolution);

    for output in switch_plan.outputs_to_disable {
        xrandr::disable_output(output);
    }

    if let Some(first) = switch_plan.outputs_to_enable.pop() {
        xrandr::enable_output(first, best_resolution, None);
        for output in switch_plan.outputs_to_enable {
            xrandr::enable_output(output, best_resolution, Some(first));
        }
    }
}
