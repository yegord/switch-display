use crate::screen::{Location, Output, Resolution, Screen};
use std::collections::HashSet;
use std::iter::Iterator;

#[derive(Debug)]
pub(crate) struct SwitchPlan<'a> {
    pub(crate) outputs_to_disable: Vec<&'a Output>,
    pub(crate) outputs_to_enable: Vec<&'a Output>,
}

pub(super) fn build_switch_plan<'a>(screen: &'a Screen) -> SwitchPlan<'a> {
    if screen
        .outputs
        .iter()
        .any(|output| output.location == Location::Internal && output.connected && output.enabled)
    {
        if screen.outputs.iter().any(|output| {
            output.location == Location::External && output.connected && output.enabled
        }) {
            SwitchPlan {
                outputs_to_disable: screen
                    .outputs
                    .iter()
                    .filter(|output| {
                        output.enabled
                            && (!output.connected || output.location == Location::Internal)
                    })
                    .collect(),
                outputs_to_enable: screen
                    .outputs
                    .iter()
                    .filter(|output| output.location == Location::External && output.connected)
                    .collect(),
            }
        } else {
            SwitchPlan {
                outputs_to_disable: screen
                    .outputs
                    .iter()
                    .filter(|output| output.enabled && !output.connected)
                    .collect(),
                outputs_to_enable: screen
                    .outputs
                    .iter()
                    .filter(|output| output.connected)
                    .collect(),
            }
        }
    } else {
        SwitchPlan {
            outputs_to_disable: screen
                .outputs
                .iter()
                .filter(|output| {
                    output.enabled && (!output.connected || output.location == Location::External)
                })
                .collect(),
            outputs_to_enable: screen
                .outputs
                .iter()
                .filter(|output| output.connected && output.location == Location::Internal)
                .collect(),
        }
    }
}

pub(super) fn choose_best_resolution(
    outputs: &[&Output],
    min_refresh_rate: Option<i32>,
) -> Option<Resolution> {
    outputs
        .iter()
        .map(|output| {
            output
                .modes
                .iter()
                .filter(|mode| {
                    min_refresh_rate
                        .is_none_or(|min_refresh_rate| mode.refresh_rate >= min_refresh_rate)
                })
                .map(|mode| mode.resolution)
                .collect::<HashSet<_>>()
        })
        .reduce(|mut acc, e| {
            acc.retain(|resolution| e.contains(resolution));
            acc
        })
        .and_then(|resolutions| resolutions.into_iter().max_by_key(Resolution::area))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen::Mode;

    #[test]
    fn when_no_outputs_nothing_must_be_switched() {
        // Arrange
        let screen = Screen {
            outputs: Vec::new(),
        };

        // Act
        let switch_plan = build_switch_plan(&screen);

        // Assert
        assert!(switch_plan.outputs_to_disable.is_empty());
        assert!(switch_plan.outputs_to_enable.is_empty());
    }

    #[test]
    fn when_nothing_is_enabled_must_enable_internal() {
        // Arrange
        let screen = Screen {
            outputs: vec![
                Output {
                    name: "eDP-1".to_string(),
                    connected: true,
                    enabled: false,
                    modes: vec![TEST_MODE],
                    location: Location::Internal,
                },
                Output {
                    name: "HDMI-1".to_string(),
                    connected: true,
                    enabled: false,
                    modes: vec![TEST_MODE],
                    location: Location::External,
                },
            ],
        };

        // Act
        let switch_plan = build_switch_plan(&screen);

        // Assert
        assert!(switch_plan.outputs_to_disable.is_empty());
        assert_eq_ref(&switch_plan.outputs_to_enable, &[&screen.outputs[0]]);
    }

    #[test]
    fn when_internal_is_enabled_must_disable_disconnected_and_enable_internal_and_external() {
        // Arrange
        let screen = Screen {
            outputs: vec![
                Output {
                    name: "eDP-1".to_string(),
                    connected: true,
                    enabled: true,
                    modes: vec![TEST_MODE],
                    location: Location::Internal,
                },
                Output {
                    name: "HDMI-1".to_string(),
                    connected: true,
                    enabled: false,
                    modes: vec![TEST_MODE],
                    location: Location::External,
                },
                Output {
                    name: "HDMI-2".to_string(),
                    connected: false,
                    enabled: true,
                    modes: vec![TEST_MODE],
                    location: Location::External,
                },
                Output {
                    name: "DP-1".to_string(),
                    connected: false,
                    enabled: false,
                    modes: vec![TEST_MODE],
                    location: Location::External,
                },
            ],
        };

        // Act
        let switch_plan = build_switch_plan(&screen);

        // Assert
        assert_eq_ref(&switch_plan.outputs_to_disable, &[&screen.outputs[2]]);
        assert_eq_ref(
            &switch_plan.outputs_to_enable,
            &[&screen.outputs[0], &screen.outputs[1]],
        );
    }

    #[test]
    fn when_internal_and_external_are_enabled_must_disable_internal_and_disconnected_and_enable_external()
     {
        // Arrange
        let screen = Screen {
            outputs: vec![
                Output {
                    name: "eDP-1".to_string(),
                    connected: true,
                    enabled: true,
                    modes: vec![TEST_MODE],
                    location: Location::Internal,
                },
                Output {
                    name: "HDMI-1".to_string(),
                    connected: true,
                    enabled: true,
                    modes: vec![TEST_MODE],
                    location: Location::External,
                },
                Output {
                    name: "HDMI-2".to_string(),
                    connected: false,
                    enabled: true,
                    modes: vec![TEST_MODE],
                    location: Location::External,
                },
                Output {
                    name: "DP-1".to_string(),
                    connected: false,
                    enabled: false,
                    modes: vec![TEST_MODE],
                    location: Location::External,
                },
            ],
        };

        // Act
        let switch_plan = build_switch_plan(&screen);

        // Assert
        assert_eq_ref(
            &switch_plan.outputs_to_disable,
            &[&screen.outputs[0], &screen.outputs[2]],
        );
        assert_eq_ref(&switch_plan.outputs_to_enable, &[&screen.outputs[1]]);
    }

    #[test]
    fn when_external_is_enabled_must_disable_external_and_disconnected_and_enable_internal() {
        // Arrange
        let screen = Screen {
            outputs: vec![
                Output {
                    name: "eDP-1".to_string(),
                    connected: true,
                    enabled: false,
                    modes: vec![TEST_MODE],
                    location: Location::Internal,
                },
                Output {
                    name: "eDP-2".to_string(),
                    connected: false,
                    enabled: true,
                    modes: vec![TEST_MODE],
                    location: Location::Internal,
                },
                Output {
                    name: "HDMI-1".to_string(),
                    connected: true,
                    enabled: true,
                    modes: vec![TEST_MODE],
                    location: Location::External,
                },
                Output {
                    name: "HDMI-2".to_string(),
                    connected: false,
                    enabled: true,
                    modes: vec![TEST_MODE],
                    location: Location::External,
                },
                Output {
                    name: "DP-1".to_string(),
                    connected: false,
                    enabled: false,
                    modes: vec![TEST_MODE],
                    location: Location::External,
                },
            ],
        };

        // Act
        let switch_plan = build_switch_plan(&screen);

        // Assert
        assert_eq_ref(
            &switch_plan.outputs_to_disable,
            &[&screen.outputs[1], &screen.outputs[2], &screen.outputs[3]],
        );
        assert_eq_ref(&switch_plan.outputs_to_enable, &[&screen.outputs[0]]);
    }

    #[test]
    fn best_resolution_for_no_outputs() {
        // Arrange
        let outputs = [];

        // Act
        let best_resolution = choose_best_resolution(&outputs, None);

        // Assert
        assert!(best_resolution.is_none());
    }

    #[test]
    fn best_resolution_for_single_output() {
        // Arrange
        let outputs = [&Output {
            name: "eDP-1".to_string(),
            connected: true,
            enabled: false,
            modes: vec![
                Mode {
                    resolution: Resolution {
                        width: 1920,
                        height: 1080,
                    },
                    refresh_rate: 60000,
                },
                Mode {
                    resolution: Resolution {
                        width: 640,
                        height: 480,
                    },
                    refresh_rate: 60000,
                },
            ],
            location: Location::Internal,
        }];

        // Act
        let best_resolution = choose_best_resolution(&outputs, None);

        // Assert
        assert_eq!(
            best_resolution,
            Some(Resolution {
                width: 1920,
                height: 1080,
            })
        );
    }

    #[test]
    fn best_resolution_for_two_outputs() {
        // Arrange
        let outputs = [
            &Output {
                name: "eDP-1".to_string(),
                connected: true,
                enabled: false,
                modes: vec![
                    Mode {
                        resolution: Resolution {
                            width: 1920,
                            height: 1080,
                        },
                        refresh_rate: 60000,
                    },
                    Mode {
                        resolution: Resolution {
                            width: 800,
                            height: 600,
                        },
                        refresh_rate: 60000,
                    },
                    Mode {
                        resolution: Resolution {
                            width: 640,
                            height: 480,
                        },
                        refresh_rate: 60000,
                    },
                ],
                location: Location::Internal,
            },
            &Output {
                name: "HDMI-1".to_string(),
                connected: true,
                enabled: false,
                modes: vec![
                    Mode {
                        resolution: Resolution {
                            width: 800,
                            height: 600,
                        },
                        refresh_rate: 30000,
                    },
                    Mode {
                        resolution: Resolution {
                            width: 640,
                            height: 480,
                        },
                        refresh_rate: 60000,
                    },
                ],
                location: Location::Internal,
            },
        ];

        // Act
        let best_resolution = choose_best_resolution(&outputs, None);

        // Assert
        assert_eq!(
            best_resolution,
            Some(Resolution {
                width: 800,
                height: 600,
            })
        );
    }

    #[test]
    fn best_resolution_for_two_outputs_with_min_refresh_rate() {
        // Arrange
        let outputs = [
            &Output {
                name: "eDP-1".to_string(),
                connected: true,
                enabled: false,
                modes: vec![
                    Mode {
                        resolution: Resolution {
                            width: 1920,
                            height: 1080,
                        },
                        refresh_rate: 60000,
                    },
                    Mode {
                        resolution: Resolution {
                            width: 800,
                            height: 600,
                        },
                        refresh_rate: 60000,
                    },
                    Mode {
                        resolution: Resolution {
                            width: 640,
                            height: 480,
                        },
                        refresh_rate: 60000,
                    },
                ],
                location: Location::Internal,
            },
            &Output {
                name: "HDMI-1".to_string(),
                connected: true,
                enabled: false,
                modes: vec![
                    Mode {
                        resolution: Resolution {
                            width: 800,
                            height: 600,
                        },
                        refresh_rate: 30000,
                    },
                    Mode {
                        resolution: Resolution {
                            width: 640,
                            height: 480,
                        },
                        refresh_rate: 60000,
                    },
                ],
                location: Location::Internal,
            },
        ];

        // Act
        let best_resolution = choose_best_resolution(&outputs, Some(50000));

        // Assert
        assert_eq!(
            best_resolution,
            Some(Resolution {
                width: 640,
                height: 480,
            })
        );
    }

    #[test]
    fn no_common_resolution() {
        // Arrange
        let outputs = [
            &Output {
                name: "eDP-1".to_string(),
                connected: true,
                enabled: false,
                modes: vec![Mode {
                    resolution: Resolution {
                        width: 1920,
                        height: 1080,
                    },
                    refresh_rate: 60000,
                }],
                location: Location::Internal,
            },
            &Output {
                name: "HDMI-1".to_string(),
                connected: true,
                enabled: false,
                modes: vec![Mode {
                    resolution: Resolution {
                        width: 800,
                        height: 600,
                    },
                    refresh_rate: 60000,
                }],
                location: Location::Internal,
            },
        ];

        // Act
        let best_resolution = choose_best_resolution(&outputs, None);

        // Assert
        assert!(best_resolution.is_none());
    }

    fn assert_eq_ref<T>(a: &[&T], b: &[&T])
    where
        T: std::fmt::Debug,
    {
        assert_eq!(a.len(), b.len(), "a={:?} b={:?}", a, b);
        a.iter().zip(b.iter()).for_each(|(x, y)| {
            assert!(
                std::ptr::eq(*x as *const T, *y as *const T),
                "x={:?} y={:?}",
                x,
                y
            );
        })
    }

    const TEST_MODE: Mode = Mode {
        resolution: Resolution {
            width: 1920,
            height: 1080,
        },
        refresh_rate: 60000,
    };
}
