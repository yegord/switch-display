#[cfg(feature = "randr")]
mod randr;
#[cfg(feature = "sway")]
mod sway;
mod utils;
#[cfg(feature = "xrandr")]
mod xrandr;

use crate::screen::{Resolution, Screen};
use crate::switch::SwitchPlan;

#[derive(Copy, Clone, Debug, clap::ValueEnum)]
pub(super) enum ScreenControllerType {
    #[cfg(feature = "xrandr")]
    Xrandr,
    #[cfg(feature = "sway")]
    Sway,
    #[cfg(feature = "randr")]
    Randr,
}

#[allow(clippy::large_enum_variant)]
enum ScreenControllerData {
    #[cfg(feature = "xrandr")]
    Xrandr,
    #[cfg(feature = "sway")]
    Sway,
    #[cfg(feature = "randr")]
    Randr(randr::RandrClient),
}

pub(super) struct ScreenController(ScreenControllerData);

impl ScreenController {
    pub(super) fn new(controller_type: ScreenControllerType) -> Self {
        Self(match controller_type {
            #[cfg(feature = "xrandr")]
            ScreenControllerType::Xrandr => ScreenControllerData::Xrandr,
            #[cfg(feature = "sway")]
            ScreenControllerType::Sway => ScreenControllerData::Sway,
            #[cfg(feature = "randr")]
            ScreenControllerType::Randr => ScreenControllerData::Randr(randr::RandrClient::new()),
        })
    }

    pub(super) fn get_outputs(&self) -> Screen {
        match &self.0 {
            #[cfg(feature = "xrandr")]
            ScreenControllerData::Xrandr => xrandr::get_outputs(),
            #[cfg(feature = "sway")]
            ScreenControllerData::Sway => sway::get_outputs(),
            #[cfg(feature = "randr")]
            ScreenControllerData::Randr(randr_client) => randr_client.get_outputs(),
        }
    }

    pub(super) fn switch_outputs(
        &mut self,
        switch_plan: &SwitchPlan,
        resolution: Option<Resolution>,
    ) {
        match &mut self.0 {
            #[cfg(feature = "xrandr")]
            ScreenControllerData::Xrandr => xrandr::switch_outputs(switch_plan, resolution),
            #[cfg(feature = "sway")]
            ScreenControllerData::Sway => sway::switch_outputs(switch_plan, resolution),
            #[cfg(feature = "randr")]
            ScreenControllerData::Randr(randr_client) => {
                randr_client.switch_outputs(switch_plan, resolution)
            }
        }
    }
}
