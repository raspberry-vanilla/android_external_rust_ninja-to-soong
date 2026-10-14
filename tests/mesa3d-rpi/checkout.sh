#!/usr/bin/env bash
# Copyright 2025 ninja-to-soong authors
# Copyright 2025 KonstaKANG
# SPDX-License-Identifier: Apache-2.0

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../../checkout.sh" https://github.com/raspberry-vanilla/android_external_mesa3d-rpi e2f593446cb90f83ba96f18b9e9010da1b8be4ef "${DEST}/external/mesa3d-rpi"

sudo apt install \
    meson-1.5 \
    python3-{mako,ply}
