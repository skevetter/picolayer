#!/usr/bin/env bash
# Picolayer installer script
# Usage: curl -fsSL https://raw.githubusercontent.com/skevetter/picolayer/main/install.sh | bash

set -eo pipefail

get_platform() {
    local os arch
    os=$(uname -s) arch=$(uname -m)

    case "$os" in
        Linux) os="unknown-linux-gnu" ;;
        Darwin) os="apple-darwin" ;;
        *) echo "Unsupported OS: $os" >&2; exit 1 ;;
    esac

    case "$arch" in
        x86_64) ;;
        aarch64|arm64) arch="aarch64" ;;
        *) echo "Unsupported arch: $arch" >&2; exit 1 ;;
    esac

    echo "$arch-$os"
}

install() {
    local version="${PICOLAYER_VERSION:-latest}"
    local install_dir="${PICOLAYER_INSTALL_DIR:-/usr/local/bin}"

    if [ "$version" = "latest" ]; then
        version=$(curl -s https://api.github.com/repos/skevetter/picolayer/releases/latest | grep -o '"tag_name": "[^"]*' | cut -d'"' -f4)
        [ -z "$version" ] && { echo "Failed to fetch latest version" >&2; exit 1; }
    fi

    local platform url tmp_dir
    platform=$(get_platform)
    url="https://github.com/skevetter/picolayer/releases/download/$version/picolayer-$platform.tar.gz"
    tmp_dir=$(mktemp -d)

    echo "Installing picolayer $version to $install_dir" >&2

    curl -fsSL "$url" | tar xz -C "$tmp_dir"
    mv "$tmp_dir/picolayer" "$install_dir/"
    chmod +x "$install_dir/picolayer"

    rm -rf "$tmp_dir"
    echo "$install_dir/picolayer"
}

install
