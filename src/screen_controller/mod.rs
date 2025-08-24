#[cfg(feature = "sway")]
mod sway;
mod utils;
#[cfg(feature = "xrandr")]
mod xrandr;
#[cfg(feature = "randr")]
mod randr;

use crate::screen::{Resolution, Screen};
use crate::switch::SwitchPlan;

#[derive(Copy, Clone, Debug, clap::ValueEnum)]
pub(super) enum ScreenController {
    #[cfg(feature = "xrandr")]
    Xrandr,
    #[cfg(feature = "sway")]
    Sway,
    #[cfg(feature = "randr")]
    Randr,
}

impl ScreenController {
    pub(super) fn get_outputs(&self) -> Screen {
        match *self {
            #[cfg(feature = "xrandr")]
            ScreenController::Xrandr => xrandr::get_outputs(),
            #[cfg(feature = "sway")]
            ScreenController::Sway => sway::get_outputs(),
            #[cfg(feature = "randr")]
            ScreenController::Randr => randr::get_outputs(),
        }
    }

    pub(super) fn switch_outputs(&self, switch_plan: &SwitchPlan, resolution: Option<Resolution>) {
        match *self {
            #[cfg(feature = "xrandr")]
            ScreenController::Xrandr => xrandr::switch_outputs(switch_plan, resolution),
            #[cfg(feature = "sway")]
            ScreenController::Sway => sway::switch_outputs(switch_plan, resolution),
            #[cfg(feature = "randr")]
            // TODO: switch using RANDR extension directly.
            ScreenController::Randr => xrandr::switch_outputs(switch_plan, resolution),
        }
    }
}
