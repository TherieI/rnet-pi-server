mod command;
mod socket;
mod error;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use socketcan::{CanDataFrame, CanFrame, EmbeddedFrame, Frame};

    use super::*;

    #[test]
    fn it_works() {
        // let x = CanFrame::Data(CanDataFrame::new(id, data));
    }
}
