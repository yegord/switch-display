pub(crate) struct Screen {
    pub(crate) outputs: Vec<Output>,
}

#[derive(Debug)]
pub(crate) struct Output {
    pub(crate) name: String,
    pub(crate) connected: bool,
    pub(crate) mode: Option<Mode>,
    pub(crate) modes: Vec<Mode>,
    pub(crate) location: Location,
}

#[derive(Debug)]
pub(crate) struct Mode {
    pub(crate) resolution: Resolution,
    pub(crate) freq: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Resolution {
    pub(crate) width: i32,
    pub(crate) height: i32,
}

impl Resolution {
    pub(crate) fn area(&self) -> i32 {
        self.width.checked_mul(self.height).expect("area should normally fit into i32")
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Location {
    Internal,
    External,
}

pub(crate) fn get_location(name: &str) -> Location {
    if name.starts_with("eDP-") || name.starts_with("LVDS-") {
        Location::Internal
    } else if name.starts_with("HDMI-") || name.starts_with("DP-") {
        Location::External
    } else {
        unreachable!("FIXME: unknown output connection location: {}", name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_location_test() {
        assert_eq!(get_location("eDP-1"), Location::Internal);
        assert_eq!(get_location("LVDS-1"), Location::Internal);
        assert_eq!(get_location("HDMI-1"), Location::External);
        assert_eq!(get_location("HDMI-2"), Location::External);
        assert_eq!(get_location("DP-1"), Location::External);
        assert_eq!(get_location("DP-2"), Location::External);
    }
}