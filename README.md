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

# Running

Install Rust and Cargo, then run from the repository root:

```sh
cargo run
```

By default, the program connects to the R-Net chair over `can0`, starts a
Bluetooth LE GATT server, and advertises as `SmartWheelchair`. On a Linux
device, make sure Bluetooth is enabled and the CAN interface is configured
before starting.

To run without a connected chair or `can0`, use the mock chair:

```sh
RNET_MOCK_CHAIR=1 cargo run
```

The mock still starts the Bluetooth LE server. Connect to `SmartWheelchair`
with a BLE client and write `DUMB`, `STOP`, `GUIDE`, or `DOCK` to its command
characteristic. `DUMB` moves forward for two seconds, then stops automatically;
`STOP` stops immediately. `GUIDE` and `DOCK` currently use the safe stop
behavior because those modes are not implemented yet. Read the status
characteristic for the current mode and behavior.

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
