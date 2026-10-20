#[derive(Debug, Clone)]
pub enum Mode {
    GuideFollowing,
    AutoDock,
    EmergencyStop,
}

#[derive(Debug, Clone)]
pub enum BluetoothMessage {
    ModeChange(Mode),
    TargetData(Vec<u8>),
}
