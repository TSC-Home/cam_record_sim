#!/bin/bash

echo "========================================"
echo "Testing cam_record_sim Installation"
echo "========================================"
echo ""

ERRORS=0
WARNINGS=0

# Test 1: Check Rust/Cargo
echo "1. Checking Rust/Cargo..."
if command -v cargo &> /dev/null; then
    echo "   ✓ Rust: $(cargo --version)"
else
    echo "   ✗ ERROR: Rust not found"
    echo "      Run: source \$HOME/.cargo/env"
    ERRORS=$((ERRORS + 1))
fi
echo ""

# Test 2: Check GStreamer
echo "2. Checking GStreamer..."
if command -v gst-launch-1.0 &> /dev/null; then
    GST_VERSION=$(gst-launch-1.0 --version 2>&1 | head -n1)
    echo "   ✓ $GST_VERSION"
else
    echo "   ✗ ERROR: GStreamer not found"
    ERRORS=$((ERRORS + 1))
fi
echo ""

# Test 3: Check bayer2rgb plugin (critical for DFK 37BUX265)
echo "3. Checking bayer2rgb plugin (required for DFK 37BUX265)..."
if gst-inspect-1.0 bayer2rgb &> /dev/null; then
    echo "   ✓ bayer2rgb plugin available"
else
    echo "   ✗ ERROR: bayer2rgb plugin not found"
    echo "      Install gstreamer-plugins-bad"
    ERRORS=$((ERRORS + 1))
fi
echo ""

# Test 4: Check v4l2-ctl
echo "4. Checking v4l-utils..."
if command -v v4l2-ctl &> /dev/null; then
    echo "   ✓ v4l-utils installed"
else
    echo "   ⚠  WARNING: v4l-utils not found"
    echo "      Recommended for camera detection"
    WARNINGS=$((WARNINGS + 1))
fi
echo ""

# Test 5: Check video group membership
echo "5. Checking video group membership..."
if groups | grep -q video; then
    echo "   ✓ User is in video group"
else
    echo "   ⚠  WARNING: User not in video group"
    echo "      Run: sudo usermod -a -G video \$USER"
    echo "      Then log out and back in"
    WARNINGS=$((WARNINGS + 1))
fi
echo ""

# Test 6: List available cameras
echo "6. Detecting cameras..."
if command -v v4l2-ctl &> /dev/null; then
    CAMERAS=$(v4l2-ctl --list-devices 2>&1)
    if [ $? -eq 0 ]; then
        echo "$CAMERAS" | grep -v "^$"

        # Check specifically for DFK 37BUX265
        if echo "$CAMERAS" | grep -q "DFK 37BUX265"; then
            echo ""
            echo "   ✓ The Imaging Source DFK 37BUX265 detected!"

            # Find the video device
            DEVICE=$(echo "$CAMERAS" | grep -A 2 "DFK 37BUX265" | grep "/dev/video" | head -1 | tr -d '\t')
            if [ -n "$DEVICE" ]; then
                echo "   Device: $DEVICE"

                # Check if it supports Bayer format
                if v4l2-ctl --device $DEVICE --list-formats 2>&1 | grep -q "RGGB"; then
                    echo "   ✓ Bayer RGGB format supported"
                fi
            fi
        fi
    else
        echo "   ⚠  Could not detect cameras (may need sudo or video group)"
    fi
else
    echo "   - v4l2-ctl not available, skipping camera detection"
fi
echo ""

# Test 7: Test GStreamer pipeline for DFK 37BUX265
if echo "$CAMERAS" | grep -q "DFK 37BUX265" && [ -n "$DEVICE" ]; then
    echo "7. Testing GStreamer pipeline with DFK 37BUX265..."
    if timeout 2 gst-launch-1.0 v4l2src device=$DEVICE num-buffers=5 ! \
        video/x-bayer,format=rggb,width=640,height=480,framerate=30/1 ! \
        bayer2rgb ! videoconvert ! fakesink 2>&1 | grep -q "Setting pipeline to NULL"; then
        echo "   ✓ GStreamer pipeline works with DFK 37BUX265"
    else
        echo "   ✗ ERROR: GStreamer pipeline failed"
        ERRORS=$((ERRORS + 1))
    fi
    echo ""
fi

# Test 8: Try to build the project
echo "8. Testing project build..."
if [ -f "Cargo.toml" ]; then
    echo "   Building project (this may take a while)..."
    if cargo build --release 2>&1 | tail -1 | grep -q "Finished"; then
        echo "   ✓ Build successful"
    else
        echo "   ✗ ERROR: Build failed"
        echo "      Run: cargo build --release"
        ERRORS=$((ERRORS + 1))
    fi
else
    echo "   - Cargo.toml not found, skipping build test"
fi
echo ""

# Summary
echo "========================================"
echo "Summary"
echo "========================================"
echo ""

if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
    echo "✓ All tests passed! Installation is complete."
    echo ""
    echo "You can now run the application:"
    echo "  cargo run --release"
elif [ $ERRORS -eq 0 ]; then
    echo "⚠  Installation mostly complete with $WARNINGS warning(s)"
    echo "   The application should work, but some features may be limited"
elif [ $ERRORS -gt 0 ]; then
    echo "✗ Installation incomplete: $ERRORS error(s), $WARNINGS warning(s)"
    echo "   Please fix the errors above before running the application"
fi

echo ""
exit $ERRORS
