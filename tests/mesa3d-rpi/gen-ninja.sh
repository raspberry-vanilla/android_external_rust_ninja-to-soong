#!/usr/bin/env bash

set -xe

[ $# -eq 3 ]
SRC_PATH="$1"
BUILD_PATH="$2"
NDK_PATH="$3"

SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"
MESON_LOCAL_PATH="${HOME}/.local/share/meson/cross"
AOSP_AARCH64="aosp-aarch64"
ANDROID_PLATFORM="35"
mkdir -p "${MESON_LOCAL_PATH}"
ANDROID_PLATFORM="${ANDROID_PLATFORM}" NDK_PATH="${NDK_PATH}" \
envsubst < "${SCRIPT_DIR}/${AOSP_AARCH64}.template" > "${MESON_LOCAL_PATH}/${AOSP_AARCH64}"

meson setup \
    --cross-file "${AOSP_AARCH64}" \
    --libdir lib64 \
    --sysconfdir=/system/vendor/etc \
    -Dbuildtype=release \
    -Dplatforms=android \
    -Dplatform-sdk-version=${ANDROID_PLATFORM} \
    -Dandroid-stub=true \
    -Dallow-fallback-for=libdrm \
    -Degl-lib-suffix=_mesa \
    -Dgles-lib-suffix=_mesa \
    -Dgallium-drivers=vc4,v3d \
    -Dvulkan-drivers=broadcom \
    -Dgbm=enabled \
    -Dgbm-backends-path=/apex/com.android.hardware.graphics.allocator.minigbm_gbm_mesa/lib64 \
    -Degl=enabled \
    -Dllvm=disabled \
    -Dcpp_rtti=false \
    -Dlmsensors=disabled \
    -Dandroid-libbacktrace=disabled \
    -Dstrip=true \
    --reconfigure \
    --wipe \
    "${BUILD_PATH}" \
    "${SRC_PATH}"
