// Copyright 2025 ninja-to-soong authors
// Copyright 2025 KonstaKANG
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Default)]
pub struct Mesa3DRpi {
    src_path: PathBuf,
}

const DEFAULTS: &str = "mesa3d-rpi-defaults";
const RAW_DEFAULTS: &str = "mesa3d-rpi-raw-defaults";

impl Mesa3DRpi {
    fn clean_gen_deps(
        &self,
        gen_deps: &Vec<PathBuf>,
        build_path: &Path,
        ctx: &Context,
    ) -> Result<(), String> {
        if !ctx.copy_to_aosp {
            return Ok(());
        }
        for gen_dep in gen_deps {
            let file_path = build_path.join(gen_dep);
            let file_extension = file_path.extension().unwrap().to_str().unwrap();
            if !["c", "cpp", "h"].contains(&file_extension) {
                continue;
            }
            write_file(
                &file_path,
                &read_file(&file_path)?
                    .lines()
                    .into_iter()
                    .filter(|line| !line.starts_with("#line"))
                    .chain(std::iter::once(""))
                    .collect::<Vec<&str>>()
                    .join("\n"),
            )?;
        }
        Ok(())
    }
    fn get_subprojects_path(&self) -> String {
        path_to_string(&self.src_path.join("subprojects"))
    }
}

impl Project for Mesa3DRpi {
    fn get_name(&self) -> &'static str {
        "mesa3d-rpi"
    }
    fn get_android_path(&self) -> Result<PathBuf, String> {
        Ok(Path::new("external").join(self.get_name()))
    }
    fn generate_package(
        &mut self,
        ctx: &Context,
        _projects_map: &ProjectsMap,
    ) -> Result<String, String> {
        self.src_path = ctx.get_android_path(self)?;
        let build_path = ctx.get_temp_path(Path::new(self.get_name()))?;
        let ndk_path = get_ndk_path(ctx)?;

        common::gen_ninja(
            &self.src_path,
            &build_path,
            vec![path_to_string(&ndk_path)],
            ctx,
            self,
        )?;

        const MESON_GENERATED: &str = "meson_generated";
        let mut package = SoongPackage::new(
            &["//visibility:public"],
            "mesa3d_rpi_licenses",
            &[
                "SPDX-license-identifier-MIT",
                "SPDX-license-identifier-Apache-2.0",
                "SPDX-license-identifier-GPL-1.0-or-later",
                "SPDX-license-identifier-GPL-2.0-only",
            ],
            &[
                "licenses/MIT",
                "licenses/Apache-2.0",
                "licenses/GPL-1.0-or-later",
                "licenses/GPL-2.0-only",
            ],
        )
        .generate(
            NinjaTargetsToGenMap::from(&[
                target!("src/gbm/backends/dri/dri_gbm.so", "dri_gbm"),
                target!("src/egl/libEGL_mesa.so", "libEGL_mesa"),
                target!(
                    "src/mesa/glapi/es1api/libGLESv1_CM_mesa.so",
                    "libGLESv1_CM_mesa"
                ),
                target!("src/mesa/glapi/es2api/libGLESv2_mesa.so", "libGLESv2_mesa"),
                target!(
                    "src/gallium/targets/dri/libgallium_dri.so",
                    "libgallium_dri"
                ),
                target!("src/gbm/libgbm_mesa.so", "libgbm_mesa"),
                target!(
                    "src/broadcom/vulkan/libvulkan_broadcom.so",
                    "vulkan.broadcom"
                ),
            ]),
            parse_build_ninja::<MesonNinjaTarget>(&build_path)?,
            &self.src_path,
            &ndk_path,
            &build_path,
            Some(MESON_GENERATED),
            self,
            ctx,
        )?;

        let gen_deps = package
            .get_dep_gen_assets()
            .into_iter()
            .filter(|include| !include.starts_with("subprojects"))
            .collect();

        // To save time it would suffice to only build the targets for generated dependencies
        // but build the whole project with NDK for sanity
        //common::ninja_build(&build_path, &gen_deps, ctx)?;
        common::ninja_build(&build_path, &Vec::new(), ctx)?;

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

        package.filter_gen_deps(MESON_GENERATED, &gen_deps)?;
        self.clean_gen_deps(&gen_deps, &build_path, ctx)?;
        common::copy_gen_deps(gen_deps, MESON_GENERATED, &build_path, ctx, self)?;

        // Remove some cflags from dri_gbm to have common defaults
        let cflags = package.get_props("dri_gbm", vec!["cflags"])?[0]
            .clone()
            .filter_default(
                SoongProp::VecStr(vec![
                    String::from("-DXXH_FORCE_ALIGN_CHECK=0"),
                    String::from("-DXXH_FORCE_MEMORY_ACCESS=0"),
                    String::from("-pthread"),
                ]),
                "cflags",
            )?
            .get_prop();

        let default_module = SoongModule::new("cc_defaults")
            .add_prop("name", SoongProp::Str(String::from(DEFAULTS)))
            .add_prop("cflags", cflags)
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
    product_variables: {{
        platform_sdk_version: {{
            cflags: ["-DANDROID_API_LEVEL=%d"],
        }},
    }},
}}
"#
            ))
            .print(ctx)
    }
    fn extend_module(&self, target: &Path, module: SoongModule) -> Result<SoongModule, String> {
        let soc_specific = |module: SoongModule| -> SoongModule {
            for lib in [
                "dri_gbm.so",
                "libEGL_mesa.so",
                "libGLESv1_CM_mesa.so",
                "libGLESv2_mesa.so",
                "libgallium_dri.so",
                "libgbm_mesa.so",
                "libvulkan_broadcom.so",
            ] {
                if target.ends_with(lib) {
                    return module.add_prop("soc_specific", SoongProp::Bool(true));
                }
            }
            module
        };
        let module = soc_specific(module);

        let relative_install_path = |module: SoongModule| -> SoongModule {
            for lib in [
                "libEGL_mesa.so",
                "libGLESv1_CM_mesa.so",
                "libGLESv2_mesa.so",
            ] {
                if target.ends_with(lib) {
                    return module
                        .add_prop("relative_install_path", SoongProp::Str(String::from("egl")));
                }
            }
            if target.ends_with("libvulkan_broadcom.so") {
                return module
                    .add_prop("relative_install_path", SoongProp::Str(String::from("hw")));
            }
            module
        };
        let module = relative_install_path(module);

        let header_libs = |module: SoongModule| -> SoongModule {
            for header_lib in [
                "libbroadcom_perfcntrs.a",
                "libbroadcom_v3d.a",
                "libdri.a",
                "libgallium.a",
                "libkmsrowinsys.a",
                "libloader.a",
                "libmesa_util.a",
                "libpipe_loader_dynamic.a",
                "libpipe_loader_static.a",
                "libswkmsdri.a",
                "libv3d-perfcntrs-v42.a",
                "libv3d-perfcntrs-v71.a",
                "libv3d-v42.a",
                "libv3d-v71.a",
                "libv3d.a",
                "libv3dv-v42.a",
                "libv3dv-v71.a",
                "libvc4.a",
                "libvulkan_instance.a",
                "libvulkan_lite_runtime.a",
                "libvulkan_wsi.a",
            ] {
                if target.ends_with(header_lib) {
                    if target.ends_with("libv3dv-v42.a")
                        || target.ends_with("libv3dv-v71.a")
                        || target.ends_with("libvulkan_lite_runtime.a")
                    {
                        return module.add_prop(
                            "header_libs",
                            SoongProp::VecStr(vec![
                                String::from("hwvulkan_headers"),
                                String::from("libdrm_headers"),
                            ]),
                        );
                    } else {
                        return module.add_prop(
                            "header_libs",
                            SoongProp::VecStr(vec![String::from("libdrm_headers")]),
                        );
                    }
                }
            }
            if target.ends_with("libEGL_mesa.so") {
                return module.add_prop(
                    "header_libs",
                    SoongProp::VecStr(vec![String::from("libnativebase_headers")]),
                );
            }
            module
        };
        let module = header_libs(module);

        let export_include_dirs = |module: SoongModule| -> SoongModule {
            if target.ends_with("libgbm_mesa.so") {
                return module.add_prop(
                    "export_include_dirs",
                    SoongProp::VecStr(vec![String::from("src/gbm/main")]),
                );
            }
            module
        };
        let module = export_include_dirs(module);

        let mut cflags = vec![
            "-Wno-asm-operand-widths",
            "-Wno-constant-conversion",
            "-Wno-enum-conversion",
            "-Wno-initializer-overrides",
            "-Wno-macro-redefined",
            "-Wno-missing-prototypes",
            "-Wno-non-virtual-dtor",
            "-Wno-pointer-arith",
        ];
        if target.ends_with("libnir.a") {
            cflags.push("-Wno-bool-conversion");
        }
        if target.ends_with("libv3dv-v42.a")
            || target.ends_with("libv3dv-v71.a")
            || target.ends_with("libvulkan_lite_runtime.a")
        {
            cflags.push("-Wno-unreachable-code-loop-increment");
        }
        if target.ends_with("lib_mesa_u_gralloc.a") {
            cflags.push("-DUSE_IMAPPER4_METADATA_API");
        }

        let mut shared_libs = Vec::new();
        if target.ends_with("libbroadcom_cle.a") {
            shared_libs.push("libexpat");
        }
        if target.ends_with("libv3dv-v42.a")
            || target.ends_with("libv3dv-v71.a")
            || target.ends_with("libvulkan_lite_runtime.a")
        {
            shared_libs.push("libnativewindow");
        }
        if target.ends_with("libdri.a")
            || target.ends_with("libgallium.a")
            || target.ends_with("libv3dv-v42.a")
            || target.ends_with("libv3dv-v71.a")
            || target.ends_with("libvulkan_lite_runtime.a")
            || target.ends_with("libvulkan_wsi.a")
        {
            shared_libs.push("libsync");
        }
        if target.ends_with("libEGL_mesa.so")
            || target.ends_with("libvulkan_broadcom.so")
            || target.ends_with("lib_mesa_u_gralloc.a")
        {
            shared_libs.push("libui");
        }
        if target.ends_with("libbroadcom_cle.a") || target.ends_with("libmesa_util.a") {
            shared_libs.push("libz");
        }

        let mut srcs = Vec::new();
        if target.ends_with("lib_mesa_u_gralloc.a") {
            srcs.push("src/util/u_gralloc/u_gralloc_imapper5_api.cpp");
        }

        module
            .add_prop("defaults", SoongProp::VecStr(vec![String::from(DEFAULTS)]))
            .extend_prop("cflags", cflags)?
            .extend_prop("shared_libs", shared_libs)?
            .extend_prop("srcs", srcs)
    }
    fn map_lib(&self, library: &Path, kind: LibraryKind) -> Option<(PathBuf, LibraryKind)> {
        if library.starts_with("src/android_stub") || !library.starts_with("src") {
            Some((PathBuf::from(file_stem(library)), kind))
        } else {
            None
        }
    }
    fn filter_cflag(&self, cflag: &str) -> bool {
        !cflag.starts_with("'") && cflag != "-fno-rtti"
    }
    fn filter_define(&self, define: &str) -> bool {
        !define.starts_with("ANDROID_API_LEVEL")
    }
    fn filter_gen_header(&self, _header: &Path) -> bool {
        false
    }
    fn filter_gen_source(&self, _source: &Path) -> bool {
        false
    }
    fn filter_include(&self, include: &Path) -> bool {
        !include.ends_with("android_stub")
            && !path_to_string(include).contains(&self.get_subprojects_path())
    }
    fn filter_link_flag(&self, flag: &str) -> bool {
        flag == "-Wl,--build-id=sha1" || flag == "-Wl,-Bsymbolic"
    }
    fn filter_target(&self, target: &Path) -> bool {
        let file_name = file_name(target);
        !file_name.ends_with(".o")
            && !file_name.ends_with(".def")
            && !file_name.contains("libdrm")
            && !file_name.contains("libexpat")
            && !target.starts_with("src/android_stub")
    }
}
