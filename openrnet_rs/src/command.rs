use socketcan::{CanDataFrame, CanFrame, EmbeddedFrame, ExtendedId};

pub(crate) const DEFAULT_JSM_SLOT: u8 = 1; // typically 1 (may need to be configurable)
const DEFAULT_PM_SLOT: u8 = 0; // consistantly 0

pub mod rnet_id {
    // https://github.com/redragonx/open-rnet/blob/main/docs/RNET_PROTOCOL_GUIDE.md
    pub const JOYSTICK: u32 = 0x02000000;
    pub const SPEED: u32 = 0x0A040000;
    pub const HORN_START: u32 = 0x0C040000;
    pub const HORN_END: u32 = 0x0C040001;
}

// converts a raw rnet command id to an extendedid
pub(crate) fn get_extended_id(raw_command_id: u32, device_slot: u8) -> Option<ExtendedId> {
    // JSM slot is 8 bits left into command
    ExtendedId::new(raw_command_id | ((device_slot as u32) << 8))
}

#[derive(Debug, Clone, Copy)]
pub enum RnetCommand {
    Joystick { x: i8, y: i8 },
    SetSpeed(u8),
    HornStart,
    HornStop,
}

impl From<RnetCommand> for CanFrame {
    fn from(value: RnetCommand) -> Self {
        match value {
            RnetCommand::Joystick { x, y } => {
                let id = get_extended_id(rnet_id::JOYSTICK, DEFAULT_JSM_SLOT)
                    .expect("Joystick should compile to a valid extended frame id");
                let data = [x.clamp(-100, 100) as u8, y.clamp(-100, 100) as u8];
                CanFrame::Data(
                    CanDataFrame::new(id, &data).expect("Joystick's CanDataFrame should be valid"),
                )
            }
            RnetCommand::SetSpeed(s) => {
                let id = get_extended_id(rnet_id::SPEED, DEFAULT_JSM_SLOT)
                    .expect("SetSpeed should compile to a valid extended frame id");
                let data = s.clamp(0, 100);
                CanFrame::Data(
                    CanDataFrame::new(id, &[data])
                        .expect("SetSpeed's CanDataFrame should be valid"),
                )
            }
            RnetCommand::HornStart => {
                let id = get_extended_id(rnet_id::HORN_START, DEFAULT_JSM_SLOT)
                    .expect("HornStart should compile to a valid extended frame id");
                CanFrame::Data(
                    CanDataFrame::new(id, &[]).expect("HornStart's CanDataFrame should be valid"),
                )
            }
            RnetCommand::HornStop => {
                let id = get_extended_id(rnet_id::HORN_END, DEFAULT_JSM_SLOT)
                    .expect("HornEnd should compile to a valid extended frame id");
                CanFrame::Data(
                    CanDataFrame::new(id, &[]).expect("HornEnd's CanDataFrame should be valid"),
                )
            }
        }
    }
}
