use crate::screen::{Resolution, Screen};
use crate::switch::SwitchPlan;

#[cfg(feature = "sway")]
mod sway;
#[cfg(feature = "xrandr")]
mod xrandr;

pub(super) enum ScreenController {
    #[cfg(feature = "xrandr")]
    Xrandr,
    #[cfg(feature = "sway")]
    Sway,
}

impl ScreenController {
    pub(super) fn get_outputs(&self) -> Screen {
        match *self {
            #[cfg(feature = "xrandr")]
            ScreenController::Xrandr => xrandr::get_outputs(),
            #[cfg(feature = "sway")]
            ScreenController::Sway => sway::get_outputs(),
        }
    }

    pub(super) fn switch_outputs(&self, switch_plan: &SwitchPlan, resolution: Option<Resolution>) {
        match *self {
            #[cfg(feature = "xrandr")]
            ScreenController::Xrandr => xrandr::switch_outputs(switch_plan, resolution),
            #[cfg(feature = "sway")]
            ScreenController::Sway => sway::switch_outputs(switch_plan, resolution),
        }
    }
}
