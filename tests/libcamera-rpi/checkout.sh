#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../../utils/checkout.sh" https://github.com/raspberry-vanilla/android_external_libcamera 85f48f41a12979aed1070c97c6d3c5f5c962f59b "${DEST}/external/libcamera"

sudo apt install \
    meson-1.5 \
    python3-{jinja2,ply,yaml}
