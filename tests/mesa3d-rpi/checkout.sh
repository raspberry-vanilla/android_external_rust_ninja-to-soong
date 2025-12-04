#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../../utils/checkout.sh" https://github.com/raspberry-vanilla/android_external_mesa3d-rpi 8656eed93d0c5f3948f9b9357f4496ad2c22d40a "${DEST}/external/mesa3d-rpi"

sudo apt install \
    meson-1.5 \
    python3-{mako,ply}
