#!/bin/bash

set -e

echo "Installing dependencies for cam_record_sim..."
echo ""

# Detect distribution
if [ -f /etc/fedora-release ]; then
    DISTRO="fedora"
elif [ -f /etc/debian_version ]; then
    DISTRO="debian"
elif [ -f /etc/arch-release ]; then
    DISTRO="arch"
elif [ -f /etc/SuSE-release ] || [ -f /etc/SUSE-release ]; then
    DISTRO="suse"
else
    DISTRO="unknown"
fi

echo "Detected distribution: $DISTRO"
echo ""

# Function to install Rust
install_rust() {
    if ! command -v cargo &> /dev/null; then
        echo "Rust not found. Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        echo "Rust installed successfully!"
        echo ""
    else
        echo "Rust is already installed."
        echo ""
    fi
}

case $DISTRO in
    fedora)
        echo "Installing Fedora dependencies..."
        sudo dnf install -y \
            gstreamer1-devel \
            gstreamer1-plugins-base-devel \
            gstreamer1-plugins-good \
            gstreamer1-plugins-bad-free \
            gstreamer1-plugin-openh264 \
            gtk4-devel \
            graphene-devel \
            glib2-devel \
            gobject-introspection-devel \
            cairo-devel \
            pango-devel \
            gdk-pixbuf2-devel \
            libusb1-devel \
            libudev-devel \
            libzip-devel \
            libuuid-devel \
            qt5-qtbase-devel \
            ninja-build \
            git \
            cmake \
            v4l-utils \
            libv4l-devel \
            python3-devel \
            python3-setuptools

        install_rust

        echo ""
        echo "Installing The Imaging Source camera support (tiscamera)..."
        echo "Note: tiscamera is optional for DFK 37BUX265 camera"
        echo "The camera works with GStreamer v4l2src + bayer2rgb without tiscamera"
        echo ""

        if [ ! -d "$HOME/tiscamera" ]; then
            echo "Cloning tiscamera repository..."
            if git clone https://github.com/TheImagingSource/tiscamera.git "$HOME/tiscamera"; then
                cd "$HOME/tiscamera"
                mkdir -p build
                cd build

                echo "Configuring tiscamera..."
                if cmake -DCMAKE_INSTALL_PREFIX=/usr -DBUILD_ARAVIS=OFF -DBUILD_GST_1_0=ON ..; then
                    echo "Building tiscamera (this may take a while)..."
                    if make -j$(nproc); then
                        echo "Installing tiscamera..."
                        sudo make install
                        sudo ldconfig
                        echo "tiscamera installed successfully!"
                    else
                        echo "Warning: tiscamera build failed, but cam_record_sim will still work with v4l2src"
                    fi
                else
                    echo "Warning: tiscamera cmake failed, but cam_record_sim will still work with v4l2src"
                fi
                cd ~
            else
                echo "Warning: Could not clone tiscamera, but cam_record_sim will still work with v4l2src"
            fi
        else
            echo "tiscamera already exists at $HOME/tiscamera"
        fi

        echo ""
        echo "The Imaging Source DFK 37BUX265 camera support ready!"
        echo "The camera will work via GStreamer v4l2src + bayer2rgb conversion"
        echo ""
        echo "Optional: For better video quality, install gstreamer1-plugins-ugly from RPM Fusion"
        echo "  sudo dnf install gstreamer1-plugins-ugly"
        ;;

    debian)
        echo "Installing Debian/Ubuntu dependencies..."
        sudo apt-get update
        sudo apt-get install -y \
            curl \
            build-essential \
            pkg-config \
            libssl-dev \
            libgstreamer1.0-dev \
            libgstreamer-plugins-base1.0-dev \
            libgstreamer-plugins-bad1.0-dev \
            gstreamer1.0-plugins-base \
            gstreamer1.0-plugins-good \
            gstreamer1.0-plugins-bad \
            gstreamer1.0-plugins-ugly \
            gstreamer1.0-libav \
            gstreamer1.0-tools \
            gstreamer1.0-x \
            gstreamer1.0-alsa \
            gstreamer1.0-gl \
            gstreamer1.0-gtk3 \
            gstreamer1.0-qt5 \
            gstreamer1.0-pulseaudio \
            libgtk-4-dev \
            libgraphene-1.0-dev \
            libglib2.0-dev \
            gobject-introspection \
            libgirepository1.0-dev \
            libcairo2-dev \
            libpango1.0-dev \
            libgdk-pixbuf-2.0-dev \
            libusb-1.0-0-dev \
            libudev-dev \
            libzip-dev \
            v4l-utils \
            libv4l-dev \
            git \
            cmake \
            ninja-build \
            python3-dev \
            python3-setuptools

        install_rust

        echo ""
        echo "Installing The Imaging Source camera support (tiscamera)..."
        echo "Note: tiscamera is optional for DFK 37BUX265 camera"
        echo "The camera works with GStreamer v4l2src + bayer2rgb without tiscamera"
        echo ""

        if [ ! -d "$HOME/tiscamera" ]; then
            echo "Cloning tiscamera repository..."
            if git clone https://github.com/TheImagingSource/tiscamera.git "$HOME/tiscamera"; then
                cd "$HOME/tiscamera"
                mkdir -p build
                cd build

                echo "Configuring tiscamera..."
                if cmake -DCMAKE_INSTALL_PREFIX=/usr -DBUILD_ARAVIS=OFF -DBUILD_GST_1_0=ON ..; then
                    echo "Building tiscamera (this may take a while)..."
                    if make -j$(nproc); then
                        echo "Installing tiscamera..."
                        sudo make install
                        sudo ldconfig
                        echo "tiscamera installed successfully!"
                    else
                        echo "Warning: tiscamera build failed, but cam_record_sim will still work with v4l2src"
                    fi
                else
                    echo "Warning: tiscamera cmake failed, but cam_record_sim will still work with v4l2src"
                fi
                cd ~
            else
                echo "Warning: Could not clone tiscamera, but cam_record_sim will still work with v4l2src"
            fi
        else
            echo "tiscamera already exists at $HOME/tiscamera"
        fi

        echo ""
        echo "The Imaging Source DFK 37BUX265 camera support ready!"
        echo "The camera will work via GStreamer v4l2src + bayer2rgb conversion"
        ;;

    arch)
        echo "Installing Arch Linux dependencies..."
        sudo pacman -Sy --noconfirm \
            base-devel \
            curl \
            gstreamer \
            gst-plugins-base \
            gst-plugins-good \
            gst-plugins-bad \
            gst-plugin-gtk \
            gtk4 \
            graphene \
            glib2 \
            gobject-introspection \
            cairo \
            pango \
            gdk-pixbuf2 \
            libusb \
            systemd-libs \
            libzip \
            git \
            cmake \
            pkg-config \
            v4l-utils \
            python \
            python-setuptools

        install_rust

        echo ""
        echo "Installing The Imaging Source camera support (tiscamera)..."
        echo "Note: tiscamera is optional for DFK 37BUX265 camera"
        echo "The camera works with GStreamer v4l2src + bayer2rgb without tiscamera"
        echo ""

        if [ ! -d "$HOME/tiscamera" ]; then
            echo "Cloning tiscamera repository..."
            if git clone https://github.com/TheImagingSource/tiscamera.git "$HOME/tiscamera"; then
                cd "$HOME/tiscamera"
                mkdir -p build
                cd build

                echo "Configuring tiscamera..."
                if cmake -DCMAKE_INSTALL_PREFIX=/usr -DBUILD_ARAVIS=OFF -DBUILD_GST_1_0=ON ..; then
                    echo "Building tiscamera (this may take a while)..."
                    if make -j$(nproc); then
                        echo "Installing tiscamera..."
                        sudo make install
                        sudo ldconfig
                        echo "tiscamera installed successfully!"
                    else
                        echo "Warning: tiscamera build failed, but cam_record_sim will still work with v4l2src"
                    fi
                else
                    echo "Warning: tiscamera cmake failed, but cam_record_sim will still work with v4l2src"
                fi
                cd ~
            else
                echo "Warning: Could not clone tiscamera, but cam_record_sim will still work with v4l2src"
            fi
        else
            echo "tiscamera already exists at $HOME/tiscamera"
        fi

        echo ""
        echo "The Imaging Source DFK 37BUX265 camera support ready!"
        echo "The camera will work via GStreamer v4l2src + bayer2rgb conversion"
        ;;

    suse)
        echo "Installing openSUSE dependencies..."
        sudo zypper install -y \
            curl \
            gcc \
            gcc-c++ \
            make \
            gstreamer-devel \
            gstreamer-plugins-base-devel \
            gstreamer-plugins-good \
            gstreamer-plugins-bad \
            gtk4-devel \
            libgraphene-devel \
            glib2-devel \
            gobject-introspection-devel \
            cairo-devel \
            pango-devel \
            gdk-pixbuf-devel \
            libusb-1_0-devel \
            libudev-devel \
            libzip-devel \
            git \
            cmake \
            ninja \
            pkg-config \
            v4l-utils \
            libv4l-devel \
            python3-devel \
            python3-setuptools

        install_rust

        echo ""
        echo "Installing The Imaging Source camera support (tiscamera)..."
        echo "Note: tiscamera is optional for DFK 37BUX265 camera"
        echo "The camera works with GStreamer v4l2src + bayer2rgb without tiscamera"
        echo ""

        if [ ! -d "$HOME/tiscamera" ]; then
            echo "Cloning tiscamera repository..."
            if git clone https://github.com/TheImagingSource/tiscamera.git "$HOME/tiscamera"; then
                cd "$HOME/tiscamera"
                mkdir -p build
                cd build

                echo "Configuring tiscamera..."
                if cmake -DCMAKE_INSTALL_PREFIX=/usr -DBUILD_ARAVIS=OFF -DBUILD_GST_1_0=ON ..; then
                    echo "Building tiscamera (this may take a while)..."
                    if make -j$(nproc); then
                        echo "Installing tiscamera..."
                        sudo make install
                        sudo ldconfig
                        echo "tiscamera installed successfully!"
                    else
                        echo "Warning: tiscamera build failed, but cam_record_sim will still work with v4l2src"
                    fi
                else
                    echo "Warning: tiscamera cmake failed, but cam_record_sim will still work with v4l2src"
                fi
                cd ~
            else
                echo "Warning: Could not clone tiscamera, but cam_record_sim will still work with v4l2src"
            fi
        else
            echo "tiscamera already exists at $HOME/tiscamera"
        fi

        echo ""
        echo "The Imaging Source DFK 37BUX265 camera support ready!"
        echo "The camera will work via GStreamer v4l2src + bayer2rgb conversion"
        ;;

    *)
        echo "Unknown distribution!"
        echo ""
        echo "Attempting to install Rust..."
        install_rust
        echo ""
        echo "Please install the following dependencies manually:"
        echo "  - GStreamer development files"
        echo "  - GStreamer plugins: base, good, bad"
        echo "  - GTK4 development files"
        echo "  - Graphene development files"
        echo "  - GLib, Cairo, Pango development files"
        echo "  - pkg-config"
        exit 1
        ;;
esac

echo ""
echo "========================================"
echo "Dependencies installed successfully!"
echo "========================================"
echo ""

# Add user to video group for camera access
echo "Configuring camera access permissions..."
if groups $USER | grep -q video; then
    echo "User already in 'video' group"
else
    echo "Adding user to 'video' group for camera access..."
    sudo usermod -a -G video $USER
    echo "✓ User added to video group"
    echo ""
    echo "⚠️  IMPORTANT: You must log out and log back in for group changes to take effect!"
    echo "    Or run: newgrp video"
fi

echo ""
echo "Verifying installation..."
echo "----------------------"

# Check Rust
if command -v cargo &> /dev/null; then
    echo "✓ Rust/Cargo: $(cargo --version)"
else
    echo "✗ Rust not found in PATH"
    echo "  Run: source \$HOME/.cargo/env"
fi

# Check GStreamer
if command -v gst-launch-1.0 &> /dev/null; then
    echo "✓ GStreamer: $(gst-launch-1.0 --version | head -n1)"
else
    echo "✗ GStreamer not found"
fi

# Check bayer2rgb plugin
if gst-inspect-1.0 bayer2rgb &> /dev/null; then
    echo "✓ bayer2rgb plugin available (required for DFK 37BUX265)"
else
    echo "✗ bayer2rgb plugin not found (required for DFK 37BUX265)"
fi

# Check v4l2-ctl
if command -v v4l2-ctl &> /dev/null; then
    echo "✓ v4l-utils installed"
else
    echo "✗ v4l-utils not found"
fi

echo ""
echo "========================================"
echo "Next Steps:"
echo "========================================"
echo ""
echo "1. If Rust was just installed, reload your shell:"
echo "   source \$HOME/.cargo/env"
echo ""
echo "2. If you were added to the video group, log out and back in"
echo "   Or run: newgrp video"
echo ""
echo "3. Check available cameras:"
echo "   v4l2-ctl --list-devices"
echo ""
echo "4. Build the project:"
echo "   cargo build --release"
echo ""
echo "5. Run the application:"
echo "   cargo run --release"
echo ""
echo "For DFK 37BUX265 camera testing:"
echo "  ./target/release/cam_record_sim list-cameras"
echo ""
