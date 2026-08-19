extern crate cc;
use glob::glob;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use std::{env, fs::File, io::BufRead, io::BufReader, io::Read, io::Write};

const AUX_C_FILES: [&str; 2] = ["./src/c/src.c", "./src/c/sjlj.s"];

const SDK_C_FILES: [&str; 14] = [
    // Syscalls
    "src/cx_stubs.S",
    "src/svc_call.s",
    "src/svc_cx_call.s",
    // stack protector
    "src/stack_protector.c",
    "src/stack_protector_init.S",
    // OS
    "src/pic.c",
    "src/os.c",
    "src/os_printf.c",
    "protocol/src/ledger_protocol.c",
    // IO
    "io/src/os_io.c",
    "io/src/os_io_default_apdu.c",
    "io/src/os_io_seph_cmd.c",
    "io/src/os_io_seph_ux.c",
    // Syscalls
    "src/syscalls.c", // It must be listed after os_io.c, because it defines weak symbols
];

#[derive(Debug, Default, PartialEq, Clone, Copy)]
enum DeviceName {
    #[default]
    NanoSPlus,
    NanoX,
    Stax,
    Flex,
    ApexP,
}

#[derive(Debug, Default)]
struct Device<'a> {
    pub name: DeviceName,
    pub c_sdk: PathBuf,
    pub target: &'a str,
    pub defines: Vec<(String, Option<String>)>,
    pub cflags: Vec<String>,
    pub glyphs_folders: Vec<PathBuf>,
    pub arm_libs: String,
    pub linker_script: String,
}

impl Device<'_> {
    /// Look up this device's static spec.
    fn spec(&self) -> &'static DeviceSpec {
        SPECS
            .iter()
            .find(|s| s.name == self.name)
            .expect("DeviceName not present in SPECS")
    }

    /// `true` when the build is using the NBGL graphics stack:
    /// always on for touchscreen devices (Stax/Flex/ApexP), and on for Nano
    /// devices only when the `nano_nbgl` feature is enabled.
    fn is_nbgl(&self) -> bool {
        !self.spec().is_nano() || env::var_os("CARGO_FEATURE_NANO_NBGL").is_some()
    }
}

impl std::fmt::Display for DeviceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceName::NanoSPlus => write!(f, "nanos2"),
            DeviceName::NanoX => write!(f, "nanox"),
            DeviceName::Stax => write!(f, "stax"),
            DeviceName::Flex => write!(f, "flex"),
            DeviceName::ApexP => write!(f, "apex_p"),
        }
    }
}

/// Per-device configuration table. Everything that differs between the five
/// supported devices is captured here so the rest of the build script can
/// stay device-agnostic.
struct DeviceSpec {
    name: DeviceName,
    /// Value of `CARGO_CFG_TARGET_OS` for this device.
    target_os: &'static str,
    target_triple: &'static str,
    /// Legacy per-device env var consulted when `LEDGER_SDK_PATH` is unset.
    env_fallback: &'static str,
    /// Subdir under `<c_sdk>/arch/` for prebuilt libs ("st33" or "st33k1").
    arch_lib_dir: &'static str,
    /// Glyph folders to feed to `icon2glyph.py` when building NBGL. For Nano
    /// devices these are only used when the `nano_nbgl` feature is enabled.
    nbgl_glyph_dirs: &'static [&'static str],
}

const SPECS: &[DeviceSpec] = &[
    DeviceSpec {
        name: DeviceName::NanoX,
        target_os: "nanox",
        target_triple: "thumbv6m-none-eabi",
        env_fallback: "NANOX_SDK",
        arch_lib_dir: "st33",
        nbgl_glyph_dirs: &["lib_nbgl/glyphs/nano"],
    },
    DeviceSpec {
        name: DeviceName::NanoSPlus,
        target_os: "nanosplus",
        target_triple: "thumbv8m.main-none-eabi",
        env_fallback: "NANOSP_SDK",
        arch_lib_dir: "st33k1",
        nbgl_glyph_dirs: &["lib_nbgl/glyphs/nano"],
    },
    DeviceSpec {
        name: DeviceName::Stax,
        target_os: "stax",
        target_triple: "thumbv8m.main-none-eabi",
        env_fallback: "STAX_SDK",
        arch_lib_dir: "st33k1",
        nbgl_glyph_dirs: &[
            "lib_nbgl/glyphs/wallet",
            "lib_nbgl/glyphs/64px",
            "lib_nbgl/glyphs/32px",
        ],
    },
    DeviceSpec {
        name: DeviceName::Flex,
        target_os: "flex",
        target_triple: "thumbv8m.main-none-eabi",
        env_fallback: "FLEX_SDK",
        arch_lib_dir: "st33k1",
        nbgl_glyph_dirs: &[
            "lib_nbgl/glyphs/wallet",
            "lib_nbgl/glyphs/64px",
            "lib_nbgl/glyphs/40px",
        ],
    },
    DeviceSpec {
        name: DeviceName::ApexP,
        target_os: "apex_p",
        target_triple: "thumbv8m.main-none-eabi",
        env_fallback: "APEX_P_SDK",
        arch_lib_dir: "st33k1",
        nbgl_glyph_dirs: &[
            "lib_nbgl/glyphs/wallet",
            "lib_nbgl/glyphs/48px",
            "lib_nbgl/glyphs/24px",
        ],
    },
];

impl DeviceSpec {
    /// Path to `devices/<target_os>/` inside this crate.
    fn config_dir(&self) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("devices")
            .join(self.target_os)
    }

    fn defines_file(&self) -> PathBuf {
        self.config_dir()
            .join(format!("c_sdk_build_{}.defines", self.target_os))
    }

    fn cflags_file(&self) -> PathBuf {
        self.config_dir()
            .join(format!("c_sdk_build_{}.cflags", self.target_os))
    }

    fn linker_script(&self) -> PathBuf {
        self.config_dir()
            .join(format!("{}_layout.ld", self.target_os))
    }

    fn is_nano(&self) -> bool {
        matches!(self.name, DeviceName::NanoX | DeviceName::NanoSPlus)
    }
}

/// Read a file as a list of lines (used for the per-device `.cflags` files).
fn read_lines(path: &Path) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()))
        .lines()
        .map(str::to_owned)
        .collect()
}

#[derive(Default)]
struct CSDKInfo {
    pub api_level: Option<u32>,
    pub target_id: String,
    pub target_name: String,
    pub c_sdk_name: String,
    pub c_sdk_hash: String,
    pub c_sdk_version: String,
}

impl CSDKInfo {
    pub fn new() -> Self {
        CSDKInfo::default()
    }
}

struct SDKBuilder<'a> {
    api_level: u32,
    gcc_toolchain: PathBuf,
    device: Device<'a>,
    cxdefines: Vec<String>,
}

impl SDKBuilder<'_> {
    pub fn new() -> Self {
        SDKBuilder {
            api_level: 0,
            gcc_toolchain: PathBuf::new(),
            device: Device::default(),
            cxdefines: Vec::new(),
        }
    }

    pub fn gcc_toolchain(&mut self) {
        // Find out where the arm toolchain is located
        let output = Command::new("arm-none-eabi-gcc")
            .arg("-print-sysroot")
            .output()
            .ok();
        let sysroot = output
            .as_ref()
            .and_then(|o| std::str::from_utf8(&o.stdout).ok())
            .unwrap_or("")
            .trim();

        let gcc_toolchain = if sysroot.is_empty() {
            // path for Debian-based systems
            String::from("/usr/lib/arm-none-eabi")
        } else {
            sysroot.to_string()
        };
        self.gcc_toolchain = PathBuf::from(gcc_toolchain);
    }

    pub fn device(&mut self) {
        let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
        let spec = SPECS
            .iter()
            .find(|s| s.target_os == target_os)
            .unwrap_or_else(|| panic!("Unsupported target_os: {target_os}"));

        let c_sdk = env::var("LEDGER_SDK_PATH")
            .or_else(|_| env::var(spec.env_fallback))
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                panic!(
                    "LEDGER_SDK_PATH or {} must be set to a Ledger C SDK checkout",
                    spec.env_fallback
                )
            });

        // Defines from the per-device .defines header. Nano devices toggle
        // between BAGL and NBGL based on the `nano_nbgl` feature; touchscreen
        // devices encode the choice directly in their .defines file.
        let mut defines = header2define(spec.defines_file().to_str().unwrap());
        let nano_nbgl = env::var_os("CARGO_FEATURE_NANO_NBGL").is_some();
        if spec.is_nano() {
            if nano_nbgl {
                defines.push(("HAVE_NBGL".into(), None));
                defines.push(("NBGL_STEP".into(), None));
                defines.push(("NBGL_USE_CASE".into(), None));
            } else {
                defines.push(("HAVE_BAGL".into(), None));
                defines.push(("HAVE_UX_FLOW".into(), None));
            }
        }

        let cflags = read_lines(&spec.cflags_file());

        let is_nbgl = !spec.is_nano() || nano_nbgl;
        let glyphs_folders: Vec<PathBuf> = if is_nbgl {
            spec.nbgl_glyph_dirs.iter().map(|d| c_sdk.join(d)).collect()
        } else {
            vec![c_sdk.join("lib_ux/glyphs")]
        };

        let arm_libs = c_sdk
            .join("arch")
            .join(spec.arch_lib_dir)
            .join("lib")
            .display()
            .to_string();

        self.device = Device {
            name: spec.name,
            c_sdk,
            target: spec.target_triple,
            defines,
            cflags,
            glyphs_folders,
            arm_libs,
            linker_script: spec.linker_script().display().to_string(),
        };

        // Export metadata for 'infos.rs'. C_SDK_GRAPHICS is set once here so
        // both bindings + cc paths agree, instead of being scattered across
        // device()/configure_lib_nbgl().
        println!("cargo:rustc-env=TARGET={}", self.device.name);
        println!(
            "cargo:rustc-env=C_SDK_GRAPHICS={}",
            if is_nbgl { "nbgl" } else { "bagl" }
        );
        println!(
            "cargo:warning={} is built",
            if is_nbgl { "NBGL" } else { "BAGL" }
        );
    }

    pub fn get_info(&mut self) {
        let sdk_info = retrieve_csdk_info(&self.device, &self.device.c_sdk);
        self.api_level = sdk_info
            .api_level
            .expect("API_LEVEL not found in Makefile.defines");
        println!("cargo:rustc-env=API_LEVEL={}", self.api_level);

        // Export the rest of the C SDK metadata for 'infos.rs'. No
        // cargo:warning= duplicates — these values land in ELF sections.
        println!("cargo:rustc-env=TARGET_ID={}", sdk_info.target_id);
        println!("cargo:rustc-env=TARGET_NAME={}", sdk_info.target_name);
        println!("cargo:rustc-env=C_SDK_NAME={}", sdk_info.c_sdk_name);
        println!("cargo:rustc-env=C_SDK_HASH={}", sdk_info.c_sdk_hash);
        println!("cargo:rustc-env=C_SDK_VERSION={}", sdk_info.c_sdk_version);
    }

    fn cxdefines(&mut self) {
        let content = fs::read_to_string(self.device.c_sdk.join("Makefile.conf.cx"))
            .expect("Could not read Makefile.conf.cx");
        // Extract the HAVE_* defines (whitespace-separated, '#'-comments stripped).
        let mut cxdefines: Vec<String> = content
            .lines()
            .filter(|line| !line.starts_with('#'))
            .flat_map(|line| {
                line.split_whitespace()
                    .filter(|word| word.starts_with("HAVE"))
            })
            .map(str::to_owned)
            .collect();
        cxdefines.push("NATIVE_LITTLE_ENDIAN".to_string());
        self.cxdefines = cxdefines;
    }

    pub fn build_c_sdk(&self) {
        let mut command = cc::Build::new();
        if env::var_os("CC").is_none() {
            command.compiler("clang");
        } else {
            // Let cc::Build determine CC from the environment variable
        }

        command
            .files(&AUX_C_FILES)
            .files(str2path(&self.device.c_sdk, &SDK_C_FILES));

        // Generate glyphs
        let glyphs_path = generate_glyphs(&self.device);

        command = command
            .include(self.gcc_toolchain.join("include"))
            .include(self.device.c_sdk.join("include"))
            .include(self.device.c_sdk.join("lib_u2f/include"))
            .include(self.device.c_sdk.join("io/include"))
            .include(self.device.c_sdk.join("io_legacy/include"))
            .include(self.device.c_sdk.join("protocol/include"))
            .include(self.device.c_sdk.join("lib_cxng/include"))
            .include(self.device.c_sdk.join("lib_ux/include"))
            .include(self.device.c_sdk.join("lib_bagl/include"))
            .include(self.device.c_sdk.join("lib_nbgl/include"))
            .include(&glyphs_path)
            .debug(true)
            .define("main", "_start")
            .clone();

        // Set the #defines
        for (define, value) in &self.device.defines {
            command.define(define.as_str(), value.as_deref());
        }

        // If the debug_csdk feature is enabled, add PRINTF defines
        if env::var_os("CARGO_FEATURE_DEBUG_CSDK").is_some() {
            command.define("HAVE_PRINTF", None);
            command.define("PRINTF", Some("mcu_usb_printf"));
        }

        // Set the CFLAGS
        for cflag in &self.device.cflags {
            command.flag(cflag);
        }

        command.target(self.device.target).include(
            self.device
                .c_sdk
                .join(format!("target/{}/include", self.device.name)),
        );

        // Configure BLE, NBGL, U2F
        for s in self.device.defines.iter() {
            if s.0 == "HAVE_IO_USB" {
                configure_lib_usb(&mut command, &self.device.c_sdk);
            }
            if s.0 == "HAVE_BLE" {
                configure_lib_ble(&mut command, &self.device.c_sdk);
            }
            if s.0 == "HAVE_NBGL" {
                configure_lib_nbgl(&mut command, &self.device.c_sdk);
            }
            if s.0 == "HAVE_BAGL" {
                let glyphs_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("glyphs");
                command
                    .include(&glyphs_path)
                    .file(glyphs_path.join("glyphs.c"));
            }
            if s.0 == "HAVE_IO_U2F" {
                configure_lib_u2f(&mut command, &self.device.c_sdk);
            }
        }

        // Configure PQC algorithms (compiled app-side)
        if env::var_os("CARGO_FEATURE_MLKEM").is_some() {
            configure_lib_mlkem(&mut command, &self.device.c_sdk);
        }
        if env::var_os("CARGO_FEATURE_MLDSA").is_some() {
            configure_lib_mldsa(&mut command, &self.device.c_sdk);
            if env::var_os("CARGO_FEATURE_MLDSA_87").is_some() {
                command.define("HAVE_MLDSA_87", None);
            }
            if env::var_os("CARGO_FEATURE_MLDSA_OPTIMIZATION").is_some() {
                command.define("HAVE_MLDSA_OPTIMIZATION", None);
            }
        }
        if env::var_os("CARGO_FEATURE_MLKEM").is_some()
            || env::var_os("CARGO_FEATURE_MLDSA").is_some()
        {
            command
                .file(self.device.c_sdk.join("src/cx_hash_iovec.c"))
                .include(&self.device.c_sdk);
        }

        // Add the defines found in the Makefile.conf.cx to our build command.
        for define in self.cxdefines.iter() {
            command.define(define, None);
        }

        // Add defines and flags specified in the LEDGER_SDK_EXTRA_DEFINES and LEDGER_SDK_EXTRA_CFLAGS environment
        // variables, if they are set.
        // This allows apps to customize the build process. Since they are added after the default includes, they can
        // override previous definitions.

        if let Ok(defs) = env::var("LEDGER_SDK_EXTRA_DEFINES") {
            for d in defs.split_whitespace() {
                if let Some((k, v)) = d.split_once('=') {
                    command.define(k, Some(v));
                } else {
                    command.define(d, None);
                }
            }
        }
        if let Ok(flags) = env::var("LEDGER_SDK_EXTRA_CFLAGS") {
            for f in flags.split_whitespace() {
                command.flag(f);
            }
        }

        /* Compile the SDK */
        command.compile("ledger-secure-sdk");

        /* Link with libc */
        let path = self.device.arm_libs.clone();
        println!("cargo:rustc-link-lib=c");
        println!("cargo:rustc-link-search={path}");
    }

    fn generate_bindings(&self) {
        let bsdk = self.device.c_sdk.display().to_string();
        let gcc_tc = self.gcc_toolchain.display().to_string();
        let args = [
            "--target=thumbv6m-none-eabi".to_string(), // exact target is irrelevant for bindings
            "-fshort-enums".to_string(),
            format!("-I{gcc_tc}/include"),
            format!("-I{bsdk}/include"),
            format!("-I{bsdk}/io/include/"),
            format!("-I{bsdk}/io_legacy/include/"),
            format!("-I{bsdk}/lib_u2f/include/"),
            format!("-I{bsdk}/lib_cxng/include/"),
        ];
        let headers = str2path(
            &self.device.c_sdk,
            &[
                "lib_cxng/include/libcxng.h", /* cxlib */
                "include/os.h",               /* syscalls */
                "include/syscalls.h",
                "include/os_ux.h",
                "lib_standard_app/swap_lib_calls.h",
                "include/os_pki.h",   /* pki */
                "include/os_hdkey.h", /* zip32 */
            ],
        );

        let mut bindings = bindgen::builder()
            .clang_args(&args)
            .prepend_enum_name(false)
            .generate_comments(false)
            .derive_default(true)
            .wrap_unsafe_ops(true)
            .use_core();

        // Target specific files
        let csdk_target_name = self.device.name.to_string();
        let header = self.device.spec().defines_file();

        bindings = bindings.clang_arg(format!("-I{bsdk}/target/{csdk_target_name}/include/"));
        bindings = bindings.header(header.to_str().unwrap());

        // SDK headers to bind against
        for header in headers.iter().map(|p| p.to_str().unwrap()) {
            bindings = bindings.header(header);
        }

        // BAGL or NBGL bindings
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        let mut include_path = "-I".to_string();
        let glyphs = out_path.join("glyphs");
        include_path += glyphs.to_str().unwrap();
        bindings = bindings.clang_args([include_path.as_str()]);
        if self.device.is_nbgl() {
            bindings = bindings.clang_args([
                format!("-I{bsdk}/lib_nbgl/include/").as_str(),
                format!("-I{bsdk}/lib_ux_nbgl/").as_str(),
            ]);
            bindings = bindings.header(
                self.device
                    .c_sdk
                    .join("lib_nbgl/include/nbgl_use_case.h")
                    .to_str()
                    .unwrap(),
            );
            if self.device.spec().is_nano() {
                bindings = bindings.clang_args(["-DHAVE_NBGL", "-DNBGL_STEP", "-DNBGL_USE_CASE"]);
            }
        } else {
            bindings = bindings.clang_args([
                format!("-I{bsdk}/lib_bagl/include/").as_str(),
                format!("-I{bsdk}/lib_ux/include/").as_str(),
            ]);
            bindings = bindings.clang_args(["-DHAVE_BAGL", "-DHAVE_UX_FLOW"]);
        }

        for define in &self.cxdefines {
            bindings = bindings.clang_arg(format!("-D{define}"));
        }

        // ML-DSA feature-gated defines for bindgen
        if env::var_os("CARGO_FEATURE_MLDSA_87").is_some() {
            bindings = bindings.clang_arg("-DHAVE_MLDSA_87");
        }

        if env::var_os("CARGO_FEATURE_MLDSA_OPTIMIZATION").is_some() {
            bindings = bindings.clang_arg("-DHAVE_MLDSA_OPTIMIZATION");
        }

        let bindings = bindings
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .generate()
            .expect("Unable to generate bindings");

        // Write the bindings to the $OUT_DIR/bindings.rs file.
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings");
    }

    fn generate_heap_size(&self) {
        let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

        // HEAP_SIZE can be either:
        //  1. A single integer (e.g. "8192")
        //  2. A comma-separated list of target:value pairs (e.g. "nanosplus: 8192, stax: 12288")
        //     where target matches CARGO_CFG_TARGET_OS.
        // If not specified, or if the current target isn't present, default to DEFAULT_HEAP_SIZE.
        const DEFAULT_HEAP_SIZE: u32 = 8192;
        let raw = env::var("HEAP_SIZE").unwrap_or_else(|_| DEFAULT_HEAP_SIZE.to_string());
        let trimmed = raw.trim();

        let heap_size_value: u32 = match trimmed.parse::<u32>() {
            Ok(v) => v, // Simple numeric form
            Err(_) => {
                // Look for a target:value entry matching the current target_os
                let mut selected: Option<u32> = None;
                for entry in trimmed.split(',') {
                    let entry = entry.trim();
                    if entry.is_empty() {
                        continue;
                    }
                    if let Some((k, v_str)) = entry.split_once(':')
                        && k.trim() == target_os
                        && let Ok(v) = v_str.trim().parse::<u32>()
                    {
                        selected = Some(v);
                        break;
                    }
                }
                selected.unwrap_or(DEFAULT_HEAP_SIZE)
            }
        };

        // the maximum heap size is 4kb less than the total RAM size for the device
        // (compare the SRAM size in the respective {target_os}_layout.ld files)
        let max_heap_size = match target_os.as_str() {
            "nanox" => 24 * 1024,
            "nanosplus" => 36 * 1024,
            "stax" => 32 * 1024,
            "flex" => 32 * 1024,
            "apex_p" => 36 * 1024,
            _ => panic!("Unknown target OS '{target_os}'"),
        };

        assert!(
            (2048..=max_heap_size).contains(&heap_size_value),
            "Invalid heap size specification '{raw}'; resolved value {heap_size_value} must be in [2048, {}] for target {}",
            max_heap_size,
            target_os
        );

        let out_dir = env::var("OUT_DIR").unwrap();
        let dest_path = Path::new(&out_dir).join("heap_size.rs");
        fs::write(
            &dest_path,
            format!("pub const HEAP_SIZE: usize = {heap_size_value};"),
        )
        .expect("Unable to write file");
    }

    fn copy_linker_script(&self) {
        let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
        // extend the library search path
        println!("cargo:rustc-link-search={}", out_dir.display());
        // copy
        std::fs::copy(
            self.device.linker_script.as_str(),
            out_dir.join(self.device.linker_script.split("/").last().unwrap()),
        )
        .unwrap();
        std::fs::copy("link.ld", out_dir.join("link.ld")).unwrap();
    }
}

fn main() {
    // Inputs that can invalidate the build. Emitted up-front so they fire
    // even if a later phase panics. `cc::Build::files(...)` already emits
    // rerun-if-changed for the SDK C files it pulls from <c_sdk>/…; we
    // list the local crate-root C sources explicitly.
    for var in [
        "LEDGER_SDK_PATH",
        "NANOX_SDK",
        "NANOSP_SDK",
        "STAX_SDK",
        "FLEX_SDK",
        "APEX_P_SDK",
        "HEAP_SIZE",
        "LEDGER_SDK_EXTRA_DEFINES",
        "LEDGER_SDK_EXTRA_CFLAGS",
        "CC",
    ] {
        println!("cargo:rerun-if-env-changed={var}");
    }
    for path in ["devices", "link.ld", "src/c/src.c", "src/c/sjlj.s"] {
        println!("cargo:rerun-if-changed={path}");
    }

    let start = Instant::now();
    let mut sdk_builder = SDKBuilder::new();
    sdk_builder.gcc_toolchain();
    sdk_builder.device();
    sdk_builder.get_info();
    sdk_builder.cxdefines();
    sdk_builder.build_c_sdk();
    sdk_builder.generate_bindings();
    sdk_builder.generate_heap_size();
    sdk_builder.copy_linker_script();
    if env::var_os("LEDGER_SDK_BUILD_TIMING").is_some() {
        println!(
            "cargo:warning=Total build.rs time: {} seconds",
            start.elapsed().as_secs()
        );
    }
}

// --------------------------------------------------
// Helper functions
// --------------------------------------------------

fn configure_lib_u2f(command: &mut cc::Build, c_sdk: &Path) {
    command.file(c_sdk.join("lib_u2f/src/u2f_transport.c"));
    command.include(c_sdk.join("lib_u2f/include"));
}

fn configure_lib_usb(command: &mut cc::Build, c_sdk: &Path) {
    command
        .file(c_sdk.join("lib_stusb/src/usbd_conf.c"))
        .file(c_sdk.join("lib_stusb/src/usbd_core.c"))
        .file(c_sdk.join("lib_stusb/src/usbd_ctlreq.c"))
        .file(c_sdk.join("lib_stusb/src/usbd_desc.c"))
        .file(c_sdk.join("lib_stusb/src/usbd_ioreq.c"))
        .file(c_sdk.join("lib_stusb/src/usbd_ledger_ccid.c"))
        .file(c_sdk.join("lib_stusb/src/usbd_ledger_cdc.c"))
        .file(c_sdk.join("lib_stusb/src/usbd_ledger_hid_kbd.c"))
        .file(c_sdk.join("lib_stusb/src/usbd_ledger_hid_u2f.c"))
        .file(c_sdk.join("lib_stusb/src/usbd_ledger_hid.c"))
        .file(c_sdk.join("lib_stusb/src/usbd_ledger_webusb.c"))
        .file(c_sdk.join("lib_stusb/src/usbd_ledger.c"))
        .include(c_sdk.join("lib_stusb/include"))
        .include(c_sdk.join("lib_stusb_impl/include"));
}

fn configure_lib_ble(command: &mut cc::Build, c_sdk: &Path) {
    command
        .file(c_sdk.join("lib_blewbxx/src/ble_cmd.c"))
        .file(c_sdk.join("lib_blewbxx/src/ble_ledger_profile_apdu.c"))
        .file(c_sdk.join("lib_blewbxx/src/ble_ledger_profile_u2f.c"))
        .file(c_sdk.join("lib_blewbxx/src/ble_ledger.c"))
        .include(c_sdk.join("lib_blewbxx/include"))
        .include(c_sdk.join("lib_blewbxx_impl/include"));
}

fn configure_lib_nbgl(command: &mut cc::Build, c_sdk: &Path) {
    let glyphs_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("glyphs");
    command
        .include(c_sdk.join("lib_nbgl/include/"))
        .include(c_sdk.join("lib_nbgl/include/fonts/"))
        .include(c_sdk.join("lib_ux_nbgl/"))
        .include(c_sdk.join("qrcode/include/"))
        .include(c_sdk.join("lib_bagl/include/"))
        .file(c_sdk.join("lib_ux_nbgl/ux.c"))
        .file(c_sdk.join("qrcode/src/qrcodegen.c"))
        .files(
            glob(c_sdk.join("lib_nbgl/src/nbgl_layout*.c").to_str().unwrap())
                .unwrap()
                .map(|x| x.unwrap())
                .collect::<Vec<PathBuf>>(),
        )
        .files(
            glob(c_sdk.join("lib_nbgl/src/nbgl_page*.c").to_str().unwrap())
                .unwrap()
                .map(|x| x.unwrap())
                .collect::<Vec<PathBuf>>(),
        )
        .files(
            glob(c_sdk.join("lib_nbgl/src/nbgl_step*.c").to_str().unwrap())
                .unwrap()
                .map(|x| x.unwrap())
                .collect::<Vec<PathBuf>>(),
        )
        .files(
            glob(
                c_sdk
                    .join("lib_nbgl/src/nbgl_use_case*.c")
                    .to_str()
                    .unwrap(),
            )
            .unwrap()
            .map(|x| x.unwrap())
            .collect::<Vec<PathBuf>>(),
        )
        .file(c_sdk.join("src/nbgl_stubs.S"))
        .include(&glyphs_path)
        .file(glyphs_path.join("glyphs.c"));
}

fn retrieve_csdk_info(device: &Device, path: &Path) -> CSDKInfo {
    let mut csdk_info = CSDKInfo::new();
    (csdk_info.api_level, csdk_info.c_sdk_name) = retrieve_makefile_infos(path);
    (csdk_info.target_id, csdk_info.target_name) = retrieve_target_file_infos(device, path);
    (csdk_info.c_sdk_hash, csdk_info.c_sdk_version) = retrieve_csdk_git_info(path);
    csdk_info
}

fn retrieve_csdk_git_info(c_sdk: &Path) -> (String, String) {
    let c_sdk_hash = match Command::new("git")
        .arg("-C")
        .arg(c_sdk)
        .arg("describe")
        .arg("--always")
        .arg("--dirty")
        .arg("--exclude")
        .arg("*")
        .arg("--abbrev=40")
        .output()
        .ok()
    {
        Some(output) => {
            if output.stdout.is_empty() {
                "None".to_string()
            } else {
                String::from_utf8(output.stdout).unwrap_or("None".to_string())
            }
        }
        None => "None".to_string(),
    };

    let c_sdk_version = match Command::new("git")
        .arg("-C")
        .arg(c_sdk)
        .arg("describe")
        .arg("--tags")
        .arg("--match")
        .arg("v[0-9]*")
        .arg("--dirty")
        .output()
        .ok()
    {
        Some(output) => {
            if output.status.success() {
                String::from_utf8(output.stdout).unwrap_or("None".to_string())
            } else {
                String::from_utf8(output.stderr).unwrap_or("None".to_string())
            }
        }
        None => "None".to_string(),
    };
    (c_sdk_hash, c_sdk_version)
}

fn retrieve_makefile_infos(c_sdk: &Path) -> (Option<u32>, String) {
    let makefile =
        File::open(c_sdk.join("Makefile.defines")).expect("Could not find Makefile.defines");
    let mut api_level: Option<u32> = None;
    for line in BufReader::new(makefile)
        .lines()
        .map(|l| l.expect("Failed to read line"))
    {
        if let Some(value) = line.split(":=").nth(1).map(str::trim)
            && line.contains("API_LEVEL")
            && api_level.is_none()
        {
            api_level = Some(value.parse().expect("API_LEVEL is not a valid u32"));
        }
        if api_level.is_some() {
            break;
        }
    }
    let makefile =
        File::open(c_sdk.join("Makefile.target")).expect("Could not find Makefile.target");
    let mut sdk_name: Option<String> = None;
    for line in BufReader::new(makefile)
        .lines()
        .map(|l| l.expect("Failed to read line"))
    {
        if let Some(value) = line.split(":=").nth(1).map(str::trim)
            && line.contains("SDK_NAME")
            && sdk_name.is_none()
        {
            sdk_name = Some(value.to_string().replace('\"', ""));
        }
        if sdk_name.is_some() {
            break;
        }
    }

    let sdk_name = sdk_name.expect("SDK_NAME not found in Makefile.target");
    (api_level, sdk_name)
}

fn retrieve_target_file_infos(device: &Device, c_sdk: &Path) -> (String, String) {
    let target_file_path = c_sdk.join(format!("target/{}/include/bolos_target.h", device.name));
    let target_file = File::open(&target_file_path).unwrap_or_else(|e| {
        panic!(
            "Could not open target file {}: {e}",
            target_file_path.display()
        )
    });
    let mut target_id: Option<String> = None;
    let mut target_name: Option<String> = None;

    for line in BufReader::new(target_file)
        .lines()
        .map(|l| l.expect("Failed to read line"))
    {
        if target_id.is_none() && line.contains("#define TARGET_ID") {
            target_id = Some(
                line.split_whitespace()
                    .nth(2)
                    .expect("Malformed `#define TARGET_ID` line in bolos_target.h")
                    .to_string(),
            );
        } else if target_name.is_none()
            && line.contains("#define TARGET_")
            && !line.contains("#define TARGET_ID")
        {
            target_name = Some(
                line.split_whitespace()
                    .nth(1)
                    .expect("Malformed `#define TARGET_*` line in bolos_target.h")
                    .to_string(),
            );
        }

        if target_id.is_some() && target_name.is_some() {
            break;
        }
    }

    let target_id = target_id.expect("TARGET_ID not found in bolos_target.h");
    let target_name = target_name.expect("TARGET_NAME not found in bolos_target.h");
    (target_id, target_name)
}

fn generate_glyphs(device: &Device) -> PathBuf {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dest_path = out_path.join("glyphs");
    if !dest_path.exists() {
        fs::create_dir_all(&dest_path).ok();
    }

    // NBGL Glyphs
    if device.is_nbgl() {
        let icon2glyph = device.c_sdk.join("lib_nbgl/tools/icon2glyph.py");

        let mut cmd = Command::new(icon2glyph.as_os_str());
        cmd.arg("--glyphcheader")
            .arg(dest_path.join("glyphs.h").as_os_str())
            .arg("--glyphcfile")
            .arg(dest_path.join("glyphs.c").as_os_str());

        if device.spec().is_nano() {
            cmd.arg("--reverse");
        }

        for folder in device.glyphs_folders.iter() {
            let mut paths: Vec<_> = std::fs::read_dir(folder)
                .unwrap()
                .map(|f| f.unwrap().path())
                .collect();
            paths.sort();
            for path in paths {
                cmd.arg(&path);
            }
        }
        let _ = cmd.output();
    }
    // BAGL Glyphs
    else {
        let icon2glyph = device.c_sdk.join("icon3.py");

        let mut cmd1 = Command::new("python3");
        cmd1.arg(icon2glyph.as_os_str());
        cmd1.arg("--glyphcheader");
        let mut cmd2 = Command::new("python3");
        cmd2.arg(icon2glyph.as_os_str());
        cmd2.arg("--glyphcfile").arg("--factorize");

        for folder in device.glyphs_folders.iter() {
            let mut paths: Vec<_> = std::fs::read_dir(folder)
                .unwrap()
                .map(|f| f.unwrap().path())
                .collect();
            paths.sort();
            for path in paths {
                cmd1.arg(&path);
                cmd2.arg(&path);
            }
        }
        let output1 = cmd1.output().unwrap();
        let output2 = cmd2.output().unwrap();

        let mut glyphs_header: File = File::create(dest_path.join("glyphs.h")).unwrap();
        glyphs_header
            .write_all(&output1.stdout)
            .expect("Failed to write glyphs.h");

        let mut glyphs_cfile = File::create(dest_path.join("glyphs.c")).unwrap();
        glyphs_cfile
            .write_all(&output2.stdout)
            .expect("Failed to write glyphs.c");
    }
    dest_path
}

/// Helper function to concatenate all paths in pathlist to c_sdk's path
fn str2path(c_sdk: &Path, pathlist: &[&str]) -> Vec<PathBuf> {
    pathlist
        .iter()
        .map(|p| c_sdk.join(p))
        .collect::<Vec<PathBuf>>()
}

/// Get all #define from a header file
fn header2define(headername: &str) -> Vec<(String, Option<String>)> {
    let mut headerfile = File::open(headername).unwrap();
    let mut header = String::new();
    headerfile.read_to_string(&mut header).unwrap();

    header
        .lines()
        .filter_map(|line| {
            if line.trim_start().starts_with("#define") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                match parts.len() {
                    2 => Some((parts[1].to_string(), None)),
                    3 => Some((parts[1].to_string(), Some(parts[2].to_string()))),
                    _ => None,
                }
            } else {
                None
            }
        })
        .collect()
}

fn configure_lib_mlkem(command: &mut cc::Build, c_sdk: &Path) {
    let src = c_sdk.join("lib_cxng/src");
    command
        .file(src.join("cx_mlkem.c"))
        .file(src.join("cx_mlkem_internal.c"))
        .file(src.join("cx_mlkem_indcpa.c"))
        .file(src.join("cx_mlkem_poly.c"))
        .file(src.join("cx_mlkem_polymat.c"))
        .file(src.join("cx_mlkem_polyvec.c"))
        .file(src.join("cx_mlkem_sample.c"))
        .file(src.join("cx_mlkem_util.c"))
        .file(src.join("cx_mlkem_params.c"))
        .include(&src);
}

fn configure_lib_mldsa(command: &mut cc::Build, c_sdk: &Path) {
    let src = c_sdk.join("lib_cxng/src");
    command
        .file(src.join("cx_mldsa.c"))
        .file(src.join("cx_mldsa_internal.c"))
        .file(src.join("cx_mldsa_lowram.c"))
        .file(src.join("cx_mldsa_packing.c"))
        .file(src.join("cx_mldsa_poly.c"))
        .file(src.join("cx_mldsa_polymat.c"))
        .file(src.join("cx_mldsa_polyvec.c"))
        .file(src.join("cx_mldsa_rounding.c"))
        .file(src.join("cx_mldsa_sample.c"))
        .file(src.join("cx_mldsa_smallpoly.c"))
        .file(src.join("cx_mldsa_util.c"))
        .file(src.join("cx_mldsa_params.c"))
        .include(&src);
}
