use crate::screen::{Resolution, Screen};
use crate::switch::SwitchPlan;

mod xrandr;

pub(super) enum ScreenController {
    Xrandr,
}

impl ScreenController {
    pub(super) fn get_outputs(&self) -> Screen {
        match *self {
            ScreenController::Xrandr => xrandr::get_outputs(),
        }
    }

    pub(super) fn switch_outputs(&self, switch_plan: &SwitchPlan, resolution: Option<Resolution>) {
        match *self {
            ScreenController::Xrandr => {
                xrandr::switch_outputs(switch_plan, resolution);
            }
        }
    }
}
