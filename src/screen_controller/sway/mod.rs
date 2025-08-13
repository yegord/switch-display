use crate::{
    screen::{Resolution, Screen},
    switch::SwitchPlan,
};

mod parsing;

pub(super) fn get_outputs() -> Screen {
    // TODO
    Screen {
        outputs: Vec::new(),
    }
}

pub(super) fn switch_outputs(switch_plan: &SwitchPlan, resolution: Option<Resolution>) {
    // TODO
}
