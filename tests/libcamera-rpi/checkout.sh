#!/usr/bin/env bash
# Copyright 2025 ninja-to-soong authors
# Copyright 2025 KonstaKANG
# SPDX-License-Identifier: Apache-2.0

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../../checkout.sh" https://github.com/raspberry-vanilla/android_external_libcamera 70c59a06934f471e0e5ae441120aee6d74aff471 "${DEST}/external/libcamera"

sudo apt install \
    meson-1.5 \
    python3-{jinja2,ply,yaml}
