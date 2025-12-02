#!/bin/bash

echo "========================================"
echo "Camera Detection Debug Script"
echo "========================================"
echo ""

# Check permissions
echo "1. Checking permissions..."
echo "Current user: $USER"
echo "Groups: $(groups)"
if groups | grep -q video; then
    echo "✓ User is in video group"
else
    echo "✗ User NOT in video group"
    echo "  Run: sudo usermod -a -G video $USER"
    echo "  Then log out and back in"
fi
echo ""

# Check /dev/video* devices
echo "2. Checking /dev/video* devices..."
if ls /dev/video* 2>/dev/null; then
    echo ""
    for dev in /dev/video*; do
        echo "Device: $dev"
        ls -l $dev

        # Check if we can read it
        if [ -r "$dev" ]; then
            echo "  ✓ Readable"
        else
            echo "  ✗ NOT readable (permission denied)"
        fi

        # Get device info with v4l2-ctl
        if command -v v4l2-ctl &> /dev/null; then
            echo "  Name: $(v4l2-ctl --device=$dev --info 2>/dev/null | grep 'Card type' | cut -d: -f2 | xargs)"

            # Check for Bayer format
            if v4l2-ctl --device=$dev --list-formats 2>/dev/null | grep -q "RGGB"; then
                echo "  ✓ Supports Bayer RGGB format (DFK camera detected!)"
            fi
        fi
        echo ""
    done
else
    echo "✗ No /dev/video* devices found!"
    echo "  Is the camera connected?"
fi
echo ""

# Check USB devices
echo "3. Checking USB devices..."
if command -v lsusb &> /dev/null; then
    echo "The Imaging Source cameras:"
    lsusb | grep -i "imaging\|199e" || echo "  No Imaging Source cameras found via USB"
    echo ""
    echo "All USB video devices:"
    lsusb | grep -i "camera\|video\|webcam" || echo "  No USB video devices found"
else
    echo "  lsusb not available"
fi
echo ""

# Test v4l2-ctl
echo "4. Detailed camera list (v4l2-ctl)..."
if command -v v4l2-ctl &> /dev/null; then
    v4l2-ctl --list-devices 2>&1
else
    echo "  v4l2-ctl not installed"
    echo "  Install: sudo apt install v4l-utils"
fi
echo ""

# Test with nokhwa (if Rust is available)
echo "5. Testing camera detection with program..."
if [ -f "./target/debug/cam_record_sim" ]; then
    echo "Debug build:"
    ./target/debug/cam_record_sim list-cameras
elif [ -f "./target/release/cam_record_sim" ]; then
    echo "Release build:"
    ./target/release/cam_record_sim list-cameras
else
    echo "  Binary not found. Build with: cargo build"
fi
echo ""

# Check GStreamer
echo "6. Testing GStreamer access..."
if command -v gst-launch-1.0 &> /dev/null; then
    # Try to access first camera
    if [ -e /dev/video0 ]; then
        echo "Testing /dev/video0 with GStreamer..."
        if timeout 2 gst-launch-1.0 v4l2src device=/dev/video0 num-buffers=1 ! fakesink 2>&1 | grep -q "Setting pipeline"; then
            echo "  ✓ GStreamer can access /dev/video0"
        else
            echo "  ✗ GStreamer cannot access /dev/video0"
        fi
    fi

    # Try DFK camera if exists
    if [ -e /dev/video2 ]; then
        echo ""
        echo "Testing /dev/video2 (potential DFK camera)..."
        if v4l2-ctl --device=/dev/video2 --list-formats 2>/dev/null | grep -q "RGGB"; then
            echo "  Detected Bayer format, testing pipeline..."
            if timeout 2 gst-launch-1.0 v4l2src device=/dev/video2 num-buffers=1 ! \
                video/x-bayer,format=rggb,width=640,height=480 ! \
                bayer2rgb ! videoconvert ! fakesink 2>&1 | grep -q "Setting pipeline"; then
                echo "  ✓ GStreamer Bayer pipeline works!"
            else
                echo "  ✗ GStreamer Bayer pipeline failed"
            fi
        fi
    fi
else
    echo "  GStreamer not installed"
fi
echo ""

echo "========================================"
echo "Recommendations:"
echo "========================================"
echo ""

# Give recommendations based on findings
if ! groups | grep -q video; then
    echo "1. Add yourself to video group:"
    echo "   sudo usermod -a -G video $USER"
    echo "   Then LOG OUT and log back in (important!)"
    echo ""
fi

if ! command -v v4l2-ctl &> /dev/null; then
    echo "2. Install v4l-utils:"
    echo "   sudo apt install v4l-utils    # Ubuntu/Debian"
    echo "   sudo dnf install v4l-utils     # Fedora"
    echo ""
fi

if [ ! -e /dev/video0 ]; then
    echo "3. No camera devices found!"
    echo "   - Check if camera is connected"
    echo "   - Check if camera is powered on"
    echo "   - Check USB connection (USB 3.0 for DFK camera)"
    echo "   - Try: lsusb to see if camera is detected"
    echo ""
fi

echo "If camera is detected but not accessible:"
echo "  - Make sure you're in video group (see above)"
echo "  - Log out and back in after adding to video group"
echo "  - Check: ls -l /dev/video* to see permissions"
echo ""
