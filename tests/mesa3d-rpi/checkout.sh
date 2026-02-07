#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../../utils/checkout.sh" https://github.com/raspberry-vanilla/android_external_mesa3d-rpi 99cc89ffc80de711aa1af00de3ea021b60ab9905 "${DEST}/external/mesa3d-rpi"

sudo apt install \
    meson-1.5 \
    python3-{mako,ply}
