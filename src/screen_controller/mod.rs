use crate::screen::{Resolution, Screen};
use crate::switch::SwitchPlan;

mod xrandr;

pub(crate) enum ScreenController {
    Xrandr,
}

impl ScreenController {
    pub(crate) fn get_outputs(&self) -> Screen {
        match *self {
            ScreenController::Xrandr => xrandr::get_outputs(),
        }
    }

    pub(crate) fn apply(&self, switch_plan: &SwitchPlan, resolution: Option<Resolution>) {
        match *self {
            ScreenController::Xrandr => {
                xrandr::apply(switch_plan, resolution);
            }
        }
    }
}
