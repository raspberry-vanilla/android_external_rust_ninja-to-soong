#!/usr/bin/env bash
# Copyright 2025 ninja-to-soong authors
# Copyright 2025 KonstaKANG
# SPDX-License-Identifier: Apache-2.0

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../../checkout.sh" https://github.com/raspberry-vanilla/android_external_mesa3d-rpi 89849b822fe4d9959e360f440c17c3e6d30d962a "${DEST}/external/mesa3d-rpi"

sudo apt install \
    meson-1.5 \
    python3-{mako,ply}
