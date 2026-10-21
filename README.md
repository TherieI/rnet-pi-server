# Setup

```sh
# Find CAN device:
ls /boot/firmware/overlays/ | grep -i mcp2515
Example output:
mcp2515-can0.dtbo
mcp2515-can1.dtbo
mcp2515.dtbo

# Edit /boot/firmware/config.txt, add:
# NOTE: when editing /boot/firmware/config.txt, dtoverlay will be one of the devices grepped earlier without the .dtbo
dtparam=spi=on
dtoverlay=mcp2515-can0,oscillator=16000000,interrupt=25
dtoverlay=spi-bcm2835-overlay

# Reboot
sudo reboot

# Bring up CAN interface
sudo ip link set can0 up type can bitrate 125000

# Verify
ip link show can0
```

# Testing

1. Install `can-utils`
```sh
sudo apt update
sudo apt install can-utils
```

2. Host a virtual CAN interface
```sh
sudo modprobe vcan
sudo ip link add dev vcan0 type vcan
sudo ip link set up vcan0
```

## Writing good rust function documentation:
```rust
/// One-sentence summary explaining what the function does (in the imperative mood).
///
/// A longer paragraph providing additional context, explaining how the function works, 
/// or detailing specific edge cases and architectural considerations.
///
/// # Examples
///
/// ```
/// use my_crate::add_together;
///
/// let result = add_together(2, 3);
/// assert_eq!(result, 5);
/// ```
///
/// # Errors
///
/// List conditions under which this function will return an `Err` variant.
///
/// # Panics
///
/// List conditions under which this function will unexpectedly panic.
///
/// # Safety
///
/// (Only if the function is `unsafe`). Explain what invariants the caller must uphold.
```
