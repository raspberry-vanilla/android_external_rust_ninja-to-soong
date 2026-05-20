#!/usr/bin/env bash
# Copyright 2025 ninja-to-soong authors
# Copyright 2025 KonstaKANG
# SPDX-License-Identifier: Apache-2.0

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../../checkout.sh" https://github.com/raspberry-vanilla/android_external_mesa3d-rpi ce4ff94462ab81eb0e6cd86c91ef71aa6641c4ac "${DEST}/external/mesa3d-rpi"

sudo apt install \
    meson-1.5 \
    python3-{mako,ply}
