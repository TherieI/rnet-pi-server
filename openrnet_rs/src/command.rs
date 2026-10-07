use socketcan::{CanDataFrame, CanFrame, EmbeddedFrame, ExtendedId};
// https://github.com/redragonx/open-rnet/blob/main/docs/RNET_PROTOCOL_GUIDE.md

pub(crate) const DEFAULT_JSM_SLOT: u8 = 1; // typically 1 (may need to be configurable)
const DEFAULT_PM_SLOT: u8 = 0; // consistantly 0

pub(crate) mod rnet_id {
    pub(crate) const JOYSTICK: u32 = 0x02000000;
}

// converts a raw rnet command id to an extendedid
pub(crate) fn get_extended_id(raw_command_id: u32, device_slot: u8) -> Option<ExtendedId> {
    ExtendedId::new(raw_command_id | ((device_slot as u32) << 8))
}

pub enum RnetCommand {
    Joystick { x: i8, y: i8 },
    SetSpeed(u8),
}

impl From<RnetCommand> for CanFrame {
    fn from(value: RnetCommand) -> Self {
        match value {
            RnetCommand::Joystick { x, y } => {
                let id = get_extended_id(rnet_id::JOYSTICK, DEFAULT_JSM_SLOT)
                    .expect("Joystick Id should be in extended id range");
                let data = [x.clamp(-100, 100) as u8, y.clamp(-100, 100) as u8];
                CanFrame::Data(CanDataFrame::new(id, &data).expect("CanDataFrame should be valid"))
            }
            RnetCommand::SetSpeed(s) => todo!(),
        }
    }
}
