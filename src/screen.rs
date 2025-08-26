#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Screen {
    pub(crate) outputs: Vec<Output>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Output {
    pub(crate) name: String,
    pub(crate) connected: bool,
    pub(crate) enabled: bool,
    pub(crate) modes: Vec<Mode>,
    pub(crate) location: Location,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Mode {
    pub(crate) resolution: Resolution,
    pub(crate) refresh_rate_millihz: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Resolution {
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl Resolution {
    pub(crate) fn area(&self) -> u64 {
        self.width as u64 * self.height as u64
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Location {
    Internal,
    External,
}

impl Location {
    pub(crate) fn from_output_name(name: &str) -> Location {
        if name.starts_with("eDP-") || name.starts_with("LVDS-") {
            Location::Internal
        } else if name.starts_with("DP-")
            || name.starts_with("DVI-")
            || name.starts_with("HDMI-")
            || name.starts_with("VGA-")
        {
            Location::External
        } else {
            unreachable!("FIXME: output with unknown location: {}", name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_from_output_name() {
        assert_eq!(Location::from_output_name("eDP-1"), Location::Internal);
        assert_eq!(Location::from_output_name("LVDS-1"), Location::Internal);
        assert_eq!(Location::from_output_name("DP-1"), Location::External);
        assert_eq!(Location::from_output_name("DVI-1"), Location::External);
        assert_eq!(Location::from_output_name("HDMI-2"), Location::External);
        assert_eq!(Location::from_output_name("VGA-1"), Location::External);
    }

    #[test]
    fn large_resolution_area() {
        assert_eq!(
            Resolution {
                width: u32::MAX,
                height: u32::MAX
            }
            .area(),
            18446744065119617025
        );
    }
}
