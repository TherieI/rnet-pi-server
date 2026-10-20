#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    GuideFollowing,
    AutoDock,
    EmergencyStop,
    DummyGo,
}

impl Mode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GuideFollowing => "GuideFollowing",
            Self::AutoDock => "AutoDock",
            Self::EmergencyStop => "EmergencyStop",
            Self::DummyGo => "DummyGo",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChairStatus {
    pub current_mode: Mode,
    pub current_behavior: &'static str,
}

impl ChairStatus {
    pub fn to_json(self) -> String {
        format!(
            r#"{{"mode":"{}","behavior":"{}"}}"#,
            self.current_mode.as_str(),
            self.current_behavior
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_mode_and_active_behavior_as_json() {
        let status = ChairStatus {
            current_mode: Mode::DummyGo,
            current_behavior: "DummyGo",
        };

        assert_eq!(
            status.to_json(),
            r#"{"mode":"DummyGo","behavior":"DummyGo"}"#
        );
    }
}

#[derive(Debug, Clone)]
pub enum BluetoothMessage {
    ModeChange(Mode),
    SensorData(Vec<u8>), //not used, but putting in a sensor controller can let us access on board and attachment sensors
    TargetData(Vec<u8>), //not used, placeholder for input data from the bluetooth receiver, will likely have data to figure out where the guide is
}
