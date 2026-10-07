# Setup

```sh
# Edit /boot/config.txt, add:
dtparam=spi=on
dtoverlay=mcp2515-can0-overlay,oscillator=16000000,interrupt=25
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