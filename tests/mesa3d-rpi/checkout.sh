#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../../utils/checkout.sh" https://github.com/raspberry-vanilla/android_external_mesa3d-rpi b0f5673c1765456e2850e4d34288cfb8d8b6540e "${DEST}/external/mesa3d-rpi"

sudo apt install \
    meson-1.5 \
    python3-{mako,ply}
