# Installation Guide

Complete installation guide for cam_record_sim with support for The Imaging Source DFK 37BUX265 camera.

## Quick Start

### Automated Installation (Recommended)

Run the installation script for your distribution:

```bash
./install-dependencies.sh
```

This will automatically:
- Detect your Linux distribution (Fedora, Ubuntu/Debian, Arch, openSUSE)
- Install all required dependencies
- Install Rust if not present
- Add you to the video group for camera access
- Verify the installation

### Supported Distributions

- **Fedora / RHEL / CentOS**
- **Ubuntu / Debian**
- **Arch Linux**
- **openSUSE**

## Manual Installation

### Ubuntu/Debian

```bash
sudo apt-get update
sudo apt-get install -y \
    curl build-essential pkg-config libssl-dev \
    libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
    libgstreamer-plugins-bad1.0-dev \
    gstreamer1.0-plugins-base gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly \
    gstreamer1.0-tools \
    libgtk-4-dev libglib2.0-dev \
    v4l-utils libv4l-dev \
    git cmake ninja-build

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# Add user to video group
sudo usermod -a -G video $USER
```

**Important**: Log out and back in after adding yourself to the video group!

### Fedora/RHEL

```bash
sudo dnf install -y \
    gstreamer1-devel gstreamer1-plugins-base-devel \
    gstreamer1-plugins-good gstreamer1-plugins-bad-free \
    gstreamer1-plugin-openh264 \
    gtk4-devel glib2-devel \
    v4l-utils libv4l-devel \
    git cmake

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# Add user to video group
sudo usermod -a -G video $USER
```

## Verification

Test your installation:

```bash
./test-installation.sh
```

This will check:
- Rust/Cargo installation
- GStreamer and required plugins (especially bayer2rgb)
- v4l-utils for camera detection
- Video group membership
- Camera detection (including DFK 37BUX265)
- GStreamer pipeline test with DFK camera
- Project build test

## Building

After installation:

```bash
cargo build --release
```

## Running

```bash
# GUI mode (default)
cargo run --release

# CLI mode - list cameras
cargo run --release -- list-cameras

# Record from camera
cargo run --release -- record --camera 2 --duration 10
```

## The Imaging Source DFK 37BUX265 Support

### Requirements

The DFK 37BUX265 requires:
1. **GStreamer** with bayer2rgb plugin (from gstreamer-plugins-bad)
2. **v4l-utils** for camera detection
3. **Video group membership** for camera access

### Verification

Check if your DFK 37BUX265 is detected:

```bash
v4l2-ctl --list-devices
```

Expected output:
```
DFK 37BUX265 (usb-0000:00:14.0-1):
	/dev/video2
	/dev/video3
```

Check if Bayer format is supported:

```bash
v4l2-ctl --device /dev/video2 --list-formats-ext
```

Expected output should include:
```
[0]: 'RGGB' (8-bit Bayer RGRG/GBGB)
    Size: Discrete 640x480
    ...
    Size: Discrete 1920x1080
    ...
    Size: Discrete 2048x1536
```

### Test GStreamer Pipeline

Test the camera with GStreamer directly:

```bash
# Test at 640x480
gst-launch-1.0 v4l2src device=/dev/video2 ! \
  video/x-bayer,format=rggb,width=640,height=480,framerate=30/1 ! \
  bayer2rgb ! videoconvert ! autovideosink

# Test at 1920x1080
gst-launch-1.0 v4l2src device=/dev/video2 ! \
  video/x-bayer,format=rggb,width=1920,height=1080,framerate=60/1 ! \
  bayer2rgb ! videoconvert ! autovideosink
```

## Troubleshooting

### "Permission denied" when accessing camera

**Solution**: Add yourself to the video group and log out/in:

```bash
sudo usermod -a -G video $USER
# Then log out and back in
```

### "No such element 'bayer2rgb'"

**Solution**: Install gstreamer-plugins-bad:

```bash
# Ubuntu/Debian
sudo apt-get install gstreamer1.0-plugins-bad

# Fedora
sudo dnf install gstreamer1-plugins-bad-free
```

### DFK 37BUX265 not detected

**Checklist**:
1. Camera is connected to USB 3.0/3.1 port (not USB 2.0)
2. Camera appears in `lsusb` output
3. User is in video group
4. v4l-utils is installed

```bash
# Check USB devices
lsusb | grep -i imaging

# Check video devices
ls -l /dev/video*

# Check group membership
groups | grep video
```

### Build fails with "error: linker 'cc' not found"

**Solution**: Install build-essential or development tools:

```bash
# Ubuntu/Debian
sudo apt-get install build-essential

# Fedora
sudo dnf groupinstall "Development Tools"
```

### GTK4 errors during build

**Solution**: Install GTK4 development files:

```bash
# Ubuntu/Debian
sudo apt-get install libgtk-4-dev

# Fedora
sudo dnf install gtk4-devel
```

## Optional: tiscamera SDK

The Imaging Source provides their own SDK (tiscamera) for advanced camera control. However, **cam_record_sim works perfectly without it** using the standard GStreamer v4l2src + bayer2rgb pipeline.

If you want to install tiscamera for advanced features:

```bash
git clone https://github.com/TheImagingSource/tiscamera.git ~/tiscamera
cd ~/tiscamera
mkdir build && cd build
cmake -DCMAKE_INSTALL_PREFIX=/usr -DBUILD_ARAVIS=OFF -DBUILD_GST_1_0=ON ..
make -j$(nproc)
sudo make install
sudo ldconfig
```

## Getting Help

If you encounter issues not covered here:

1. Run the test script: `./test-installation.sh`
2. Check the [README.md](README.md) for more details
3. Open an issue on GitHub with the output of the test script
