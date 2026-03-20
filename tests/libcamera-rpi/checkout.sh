#!/usr/bin/env bash
# Copyright 2025 ninja-to-soong authors
# Copyright 2025 KonstaKANG
# SPDX-License-Identifier: Apache-2.0

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../../utils/checkout.sh" https://github.com/raspberry-vanilla/android_external_libcamera 77fcff3ea8eae6ea6427ea8308a9fd47f27c6dad "${DEST}/external/libcamera"

sudo apt install \
    meson-1.5 \
    python3-{jinja2,ply,yaml}
