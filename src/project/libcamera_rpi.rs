// Copyright 2025 ninja-to-soong authors
// Copyright 2025 KonstaKANG
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Default)]
pub struct LibcameraRpi {
    src_path: PathBuf,
}

const DEFAULTS: &str = "libcamera-rpi-defaults";
const RAW_DEFAULTS: &str = "libcamera-rpi-raw-defaults";

impl Project for LibcameraRpi {
    fn get_name(&self) -> &'static str {
        "libcamera-rpi"
    }
    fn get_android_path(&self) -> Result<PathBuf, String> {
        Ok(PathBuf::from("external/libcamera"))
    }
    fn get_test_path(&self, ctx: &Context) -> Result<PathBuf, String> {
        Ok(ctx.test_path.join(self.get_name()))
    }
    fn generate_package(
        &mut self,
        ctx: &Context,
        _projects_map: &ProjectsMap,
    ) -> Result<String, String> {
        self.src_path = ctx.get_android_path(self)?;
        let ndk_path = get_ndk_path(&ctx.temp_path, ctx)?;
        let build_path = ctx.temp_path.join(self.get_name());

        if !ctx.skip_gen_ninja {
            execute_cmd!(
                "bash",
                [
                    &path_to_string(self.get_test_path(ctx)?.join("gen-ninja.sh")),
                    &path_to_string(&self.src_path),
                    &path_to_string(&build_path),
                    &path_to_string(&ndk_path)
                ]
            )?;
        }
        common::ninja_build(&build_path, &Vec::new(), ctx)?;

        const MESON_GENERATED: &str = "meson_generated";
        let mut package = SoongPackage::new(
            &["//visibility:public"],
            "libcamera_rpi_licenses",
            &["SPDX-license-identifier-LGPL-2.1-or-later"],
            &[],
        )
        .generate(
            NinjaTargetsToGenMap::from(&[
                target!("src/libcamera/libcamera.so", "libcamera"),
                target!("src/android/libcamera-hal.so", "camera.libcamera"),
                target!("src/libcamera/base/libcamera-base.so", "libcamera-base"),
                target!("src/ipa/rpi/pisp/ipa_rpi_pisp.so", "ipa_rpi_pisp"),
                target!("src/ipa/rpi/vc4/ipa_rpi_vc4.so", "ipa_rpi_vc4"),
            ]),
            parse_build_ninja::<MesonNinjaTarget>(&build_path)?,
            &self.src_path,
            &ndk_path,
            &build_path,
            Some(MESON_GENERATED),
            self,
            ctx,
        )?;

        // Clean subprojects to prevent Soong from parsing blueprints that came with them
        if !ctx.skip_gen_ninja {
            execute_cmd!(
                "git",
                [
                    "-C",
                    &path_to_string(&self.src_path),
                    "clean",
                    "-xffd",
                    "subprojects/*"
                ]
            )?;
        }

        let gen_deps = package
            .get_gen_deps()
            .into_iter()
            .filter(|include| !include.starts_with("subprojects"))
            .collect();
        package.filter_local_include_dirs(MESON_GENERATED, &gen_deps)?;
        common::clean_gen_deps(&gen_deps, &build_path, ctx)?;
        common::copy_gen_deps(gen_deps, MESON_GENERATED, &build_path, ctx, self)?;

        // HACK: Remove one cflag from metadata.a to have common defaults and avoid
        // patching libcamera source between NDK (ninja-to-soong) and Android builds
        let prop_cflags = SoongNamedProp::get_prop(
            &package.get_props(
                "libcamera-rpi_src_android_libcamera_metadata_a",
                vec!["cflags"],
            )?[0],
        );
        let mut cflags = match prop_cflags {
            SoongProp::VecStr(t) => t,
            _ => Vec::new(),
        };
        cflags.retain(|a| a != "-Wno-shadow");

        let default_module = SoongModule::new("cc_defaults")
            .add_prop("name", SoongProp::Str(String::from(DEFAULTS)))
            .add_prop("cflags", SoongProp::VecStr(cflags))
            .add_prop(
                "defaults",
                SoongProp::VecStr(vec![String::from(RAW_DEFAULTS)]),
            );

        package
            .add_module(default_module)
            .add_raw_suffix(&format!(
                r#"
cc_defaults {{
    name: "{RAW_DEFAULTS}",
    rtti: true,
}}
"#
            ))
            .print(ctx)
    }
    fn extend_module(&self, target: &Path, module: SoongModule) -> Result<SoongModule, String> {
        let soc_specific = |module: SoongModule| -> SoongModule {
            for lib in [
                "libcamera.so",
                "libcamera-hal.so",
                "libcamera-base.so",
                "ipa_rpi_pisp.so",
                "ipa_rpi_vc4.so",
            ] {
                if target.ends_with(lib) {
                    return module.add_prop("soc_specific", SoongProp::Bool(true));
                }
            }
            module
        };
        let module = soc_specific(module);

        let relative_install_path = |module: SoongModule| -> SoongModule {
            if target.ends_with("ipa_rpi_pisp.so") || target.ends_with("ipa_rpi_vc4.so") {
                return module.add_prop(
                    "relative_install_path",
                    SoongProp::Str(String::from("libcamera/ipa")),
                );
            }
            if target.ends_with("libcamera-hal.so") {
                return module
                    .add_prop("relative_install_path", SoongProp::Str(String::from("hw")));
            }
            module
        };
        let module = relative_install_path(module);

        let header_libs = |module: SoongModule| -> SoongModule {
            if target.ends_with("libcamera.so") || target.ends_with("ipa_rpi_pisp.so") {
                return module.add_prop(
                    "header_libs",
                    SoongProp::VecStr(vec![String::from("libpisp_headers")]),
                );
            }
            module
        };
        let module = header_libs(module);

        // From generated config.h
        let mut cflags = vec![
            "-DHAVE_BACKTRACE",
            "-DHAVE_CLOSE_RANGE",
            "-DHAVE_FILE_SEALS",
            "-DHAVE_IPA_PUBKEY",
            "-DHAVE_LOCALE_T",
            "-DHAVE_MEMFD_CREATE",
            "-DHAVE_POSIX_IOCTL",
            "-DIPA_CONFIG_DIR=\\\"/vendor/etc/libcamera/ipa:/usr/local/share/libcamera/ipa\\\"",
            "-DIPA_MODULE_DIR=\\\"/usr/local/lib/libcamera/ipa\\\"",
            "-DIPA_PROXY_DIR=\\\"/usr/local/libexec/libcamera\\\"",
            "-DLIBCAMERA_DATA_DIR=\\\"/usr/local/share/libcamera\\\"",
            "-DLIBCAMERA_SYSCONF_DIR=\\\"/vendor/etc/libcamera\\\"",
        ];
        if target.ends_with("libcamera-hal.so") {
            cflags.push("-DHAVE_LIBJPEG");
        }

        let mut shared_libs = Vec::new();
        if target.ends_with("libcamera-hal.so") {
            shared_libs.push("libexif");
            shared_libs.push("libjpeg");
            shared_libs.push("libyuv_chromium");
        }

        let mut static_libs = Vec::new();
        if target.ends_with("libcamera.so") {
            static_libs.push("libyaml");
        }

        let mut srcs = Vec::new();
        if target.ends_with("libcamera-hal.so") {
            srcs.push("src/android/jpeg/encoder_libjpeg.cpp");
            srcs.push("src/android/jpeg/exif.cpp");
            srcs.push("src/android/jpeg/post_processor_jpeg.cpp");
            srcs.push("src/android/jpeg/thumbnailer.cpp");
        }

        module
            .add_prop("defaults", SoongProp::VecStr(vec![String::from(DEFAULTS)]))
            .extend_prop("cflags", cflags)?
            .extend_prop("shared_libs", shared_libs)?
            .extend_prop("static_libs", static_libs)?
            .extend_prop("srcs", srcs)
    }
    fn map_lib(&self, library: &Path) -> Option<PathBuf> {
        if !library.starts_with("src") {
            Some(PathBuf::from(file_stem(library)))
        } else {
            None
        }
    }
    fn filter_cflag(&self, cflag: &str) -> bool {
        cflag != "-include" && !cflag.ends_with("config.h")
    }
    fn filter_define(&self, define: &str) -> bool {
        define != "YAML_DECLARE_STATIC"
    }
    fn filter_include(&self, include: &Path) -> bool {
        let inc = path_to_string(include);
        let subprojects = self.src_path.join("subprojects");
        !inc.contains(&path_to_string(&subprojects))
    }
    fn filter_link_flag(&self, _flag: &str) -> bool {
        false
    }
    fn filter_gen_header(&self, _header: &Path) -> bool {
        false
    }
    fn filter_lib(&self, lib: &str) -> bool {
        !lib.contains("libatomic") && !lib.contains("libyaml") && !lib.contains("libyuv")
    }
    fn filter_target(&self, target: &Path) -> bool {
        let file_name = file_name(target);
        !file_name.ends_with(".o")
            && !file_name.contains("libpisp")
            && !file_name.contains("libyaml")
            && !file_name.contains("libyuv")
    }
}
