extern crate cc;
use glob::glob;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use std::{env, fs::File, io::BufRead, io::BufReader, io::Read, io::Write};

const AUX_C_FILES: [&str; 2] = ["./src/c/src.c", "./src/c/sjlj.s"];

const SDK_C_FILES: [&str; 13] = [
    "src/pic.c",
    "src/checks.c",
    "src/cx_stubs.S",
    "src/os.c",
    "src/svc_call.s",
    "src/svc_cx_call.s",
    "src/os_printf.c",
    "protocol/src/ledger_protocol.c",
    "io/src/os_io.c",
    "io/src/os_io_default_apdu.c",
    "io/src/os_io_seph_cmd.c",
    "io/src/os_io_seph_ux.c",
    "src/syscalls.c",
];

#[derive(Debug, Default, PartialEq)]
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

#[derive(Debug)]
enum SDKBuildError {
    UnsupportedDevice,
    InvalidAPILevel,
    MissingSDKName,
    MissingSDKPath,
    TargetFileNotFound,
    MissingTargetId,
    MissingTargetName,
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

    pub fn gcc_toolchain(&mut self) -> Result<(), SDKBuildError> {
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
            format!("{sysroot}")
        };
        self.gcc_toolchain = PathBuf::from(gcc_toolchain);
        Ok(())
    }

    pub fn device(&mut self) -> Result<(), SDKBuildError> {
        println!("cargo:rerun-if-env-changed=LEDGER_SDK_PATH");
        // determine device
        self.device = match env::var_os("CARGO_CFG_TARGET_OS")
            .unwrap()
            .to_str()
            .unwrap()
        {
            "nanosplus" => Device {
                name: DeviceName::NanoSPlus,
                c_sdk: match env::var("LEDGER_SDK_PATH").or_else(|_| env::var("NANOSP_SDK")) {
                    Ok(path) => PathBuf::from(path),
                    Err(_) => return Err(SDKBuildError::MissingSDKPath),
                },
                target: "thumbv8m.main-none-eabi",
                defines: {
                    let mut v = header2define(
                        format!(
                            "{}/devices/nanosplus/c_sdk_build_nanosplus.defines",
                            env!("CARGO_MANIFEST_DIR")
                        )
                        .as_str(),
                    );
                    if env::var_os("CARGO_FEATURE_NANO_NBGL").is_some() {
                        println!("cargo:warning=NBGL is built");
                        v.push((String::from("HAVE_NBGL"), None));
                        v.push((String::from("NBGL_STEP"), None));
                        v.push((String::from("NBGL_USE_CASE"), None));
                    } else {
                        println!("cargo:warning=BAGL is built");
                        println!("cargo:rustc-env=C_SDK_GRAPHICS={}", "bagl");
                        v.push((String::from("HAVE_BAGL"), None));
                        v.push((String::from("HAVE_UX_FLOW"), None));
                    }
                    v
                },
                cflags: {
                    let m_path = format!(
                        "{}/devices/nanosplus/c_sdk_build_nanosplus.cflags",
                        env!("CARGO_MANIFEST_DIR")
                    );
                    let f = File::open(m_path)
                        .expect("Failed to open c_sdk_build_nanosplus.cflags file");
                    let reader = BufReader::new(f);
                    reader
                        .lines()
                        .filter_map(|line| line.ok())
                        .collect::<Vec<String>>()
                },
                glyphs_folders: Vec::new(),
                arm_libs: Default::default(),
                linker_script: format!(
                    "{}/devices/nanosplus/nanosplus_layout.ld",
                    env!("CARGO_MANIFEST_DIR")
                ),
            },
            "nanox" => Device {
                name: DeviceName::NanoX,
                c_sdk: match env::var("LEDGER_SDK_PATH").or_else(|_| env::var("NANOX_SDK")) {
                    Ok(path) => PathBuf::from(path),
                    Err(_) => return Err(SDKBuildError::MissingSDKPath),
                },
                target: "thumbv6m-none-eabi",
                defines: {
                    let mut v = header2define(
                        format!(
                            "{}/devices/nanox/c_sdk_build_nanox.defines",
                            env!("CARGO_MANIFEST_DIR")
                        )
                        .as_str(),
                    );
                    if env::var_os("CARGO_FEATURE_NANO_NBGL").is_some() {
                        println!("cargo:warning=NBGL is built");
                        v.push((String::from("HAVE_NBGL"), None));
                        v.push((String::from("NBGL_STEP"), None));
                        v.push((String::from("NBGL_USE_CASE"), None));
                    } else {
                        println!("cargo:warning=BAGL is built");
                        println!("cargo:rustc-env=C_SDK_GRAPHICS={}", "bagl");
                        v.push((String::from("HAVE_BAGL"), None));
                        v.push((String::from("HAVE_UX_FLOW"), None));
                    }
                    v
                },
                cflags: {
                    let m_path = format!(
                        "{}/devices/nanox/c_sdk_build_nanox.cflags",
                        env!("CARGO_MANIFEST_DIR")
                    );
                    let f =
                        File::open(m_path).expect("Failed to open c_sdk_build_nanox.cflags file");
                    let reader = BufReader::new(f);
                    reader
                        .lines()
                        .filter_map(|line| line.ok())
                        .collect::<Vec<String>>()
                },
                glyphs_folders: Vec::new(),
                arm_libs: Default::default(),
                linker_script: format!(
                    "{}/devices/nanox/nanox_layout.ld",
                    env!("CARGO_MANIFEST_DIR")
                ),
            },
            "stax" => Device {
                name: DeviceName::Stax,
                c_sdk: match env::var("LEDGER_SDK_PATH").or_else(|_| env::var("STAX_SDK")) {
                    Ok(path) => PathBuf::from(path),
                    Err(_) => return Err(SDKBuildError::MissingSDKPath),
                },
                target: "thumbv8m.main-none-eabi",
                defines: header2define(
                    format!(
                        "{}/devices/stax/c_sdk_build_stax.defines",
                        env!("CARGO_MANIFEST_DIR")
                    )
                    .as_str(),
                ),
                cflags: {
                    let m_path = format!(
                        "{}/devices/stax/c_sdk_build_stax.cflags",
                        env!("CARGO_MANIFEST_DIR")
                    );
                    let f =
                        File::open(m_path).expect("Failed to open c_sdk_build_stax.cflags file");
                    let reader = BufReader::new(f);
                    reader
                        .lines()
                        .filter_map(|line| line.ok())
                        .collect::<Vec<String>>()
                },
                glyphs_folders: Vec::new(),
                arm_libs: Default::default(),
                linker_script: format!(
                    "{}/devices/stax/stax_layout.ld",
                    env!("CARGO_MANIFEST_DIR")
                ),
            },
            "flex" => Device {
                name: DeviceName::Flex,
                c_sdk: match env::var("LEDGER_SDK_PATH").or_else(|_| env::var("FLEX_SDK")) {
                    Ok(path) => PathBuf::from(path),
                    Err(_) => return Err(SDKBuildError::MissingSDKPath),
                },
                target: "thumbv8m.main-none-eabi",
                defines: header2define(
                    format!(
                        "{}/devices/flex/c_sdk_build_flex.defines",
                        env!("CARGO_MANIFEST_DIR")
                    )
                    .as_str(),
                ),
                cflags: {
                    let m_path = format!(
                        "{}/devices/flex/c_sdk_build_flex.cflags",
                        env!("CARGO_MANIFEST_DIR")
                    );
                    let f =
                        File::open(m_path).expect("Failed to open c_sdk_build_flex.cflags file");
                    let reader = BufReader::new(f);
                    reader
                        .lines()
                        .filter_map(|line| line.ok())
                        .collect::<Vec<String>>()
                },
                glyphs_folders: Vec::new(),
                arm_libs: Default::default(),
                linker_script: format!(
                    "{}/devices/flex/flex_layout.ld",
                    env!("CARGO_MANIFEST_DIR")
                ),
            },
            "apex_p" => Device {
                name: DeviceName::ApexP,
                c_sdk: match env::var("LEDGER_SDK_PATH").or_else(|_| env::var("APEX_P_SDK")) {
                    Ok(path) => PathBuf::from(path),
                    Err(_) => return Err(SDKBuildError::MissingSDKPath),
                },
                target: "thumbv8m.main-none-eabi",
                defines: header2define(
                    format!(
                        "{}/devices/apex_p/c_sdk_build_apex_p.defines",
                        env!("CARGO_MANIFEST_DIR")
                    )
                    .as_str(),
                ),
                cflags: {
                    let m_path = format!(
                        "{}/devices/apex_p/c_sdk_build_apex_p.cflags",
                        env!("CARGO_MANIFEST_DIR")
                    );
                    let f =
                        File::open(m_path).expect("Failed to open c_sdk_build_apex_p.cflags file");
                    let reader = BufReader::new(f);
                    reader
                        .lines()
                        .filter_map(|line| line.ok())
                        .collect::<Vec<String>>()
                },
                glyphs_folders: Vec::new(),
                arm_libs: Default::default(),
                linker_script: format!(
                    "{}/devices/apex_p/apex_p_layout.ld",
                    env!("CARGO_MANIFEST_DIR")
                ),
            },
            _ => {
                return Err(SDKBuildError::UnsupportedDevice);
            }
        };

        // set glyphs folders
        match self.device.name {
            DeviceName::Flex => {
                self.device
                    .glyphs_folders
                    .push(self.device.c_sdk.join("lib_nbgl/glyphs/wallet"));
                self.device
                    .glyphs_folders
                    .push(self.device.c_sdk.join("lib_nbgl/glyphs/64px"));
                self.device
                    .glyphs_folders
                    .push(self.device.c_sdk.join("lib_nbgl/glyphs/40px"));
            }
            DeviceName::Stax => {
                self.device
                    .glyphs_folders
                    .push(self.device.c_sdk.join("lib_nbgl/glyphs/wallet"));
                self.device
                    .glyphs_folders
                    .push(self.device.c_sdk.join("lib_nbgl/glyphs/64px"));
                self.device
                    .glyphs_folders
                    .push(self.device.c_sdk.join("lib_nbgl/glyphs/32px"));
            }
            DeviceName::ApexP => {
                self.device
                    .glyphs_folders
                    .push(self.device.c_sdk.join("lib_nbgl/glyphs/wallet"));
                self.device
                    .glyphs_folders
                    .push(self.device.c_sdk.join("lib_nbgl/glyphs/48px"));
                self.device
                    .glyphs_folders
                    .push(self.device.c_sdk.join("lib_nbgl/glyphs/24px"));
            }
            DeviceName::NanoSPlus | DeviceName::NanoX => {
                if env::var_os("CARGO_FEATURE_NANO_NBGL").is_some() {
                    self.device
                        .glyphs_folders
                        .push(self.device.c_sdk.join("lib_nbgl/glyphs/nano"));
                } else {
                    self.device
                        .glyphs_folders
                        .push(self.device.c_sdk.join("lib_ux/glyphs"));
                }
            }
        }

        // Set ARM pre-compiled libraries path
        self.device.arm_libs = match self.device.name {
            DeviceName::NanoX => {
                let mut path = self.device.c_sdk.display().to_string();
                path.push_str("/arch/st33/lib");
                path
            }
            DeviceName::NanoSPlus | DeviceName::Flex | DeviceName::Stax | DeviceName::ApexP => {
                let mut path = self.device.c_sdk.display().to_string();
                path.push_str("/arch/st33k1/lib");
                path
            }
        };

        // export TARGET into env for 'infos.rs'
        println!("cargo:rustc-env=TARGET={}", self.device.name);
        println!("cargo:warning=Device is {:?}", self.device.name);
        Ok(())
    }

    pub fn get_info(&mut self) -> Result<(), SDKBuildError> {
        // Retrieve the C SDK information
        let sdk_info = retrieve_csdk_info(&self.device, &self.device.c_sdk)?;
        match sdk_info.api_level {
            Some(api_level) => {
                self.api_level = api_level;
                // Export api level into env for 'infos.rs'
                println!("cargo:rustc-env=API_LEVEL={}", self.api_level);
                println!("cargo:warning=API_LEVEL is {}", self.api_level);
            }
            None => return Err(SDKBuildError::InvalidAPILevel),
        }

        // Export other SDK infos into env for 'infos.rs'
        println!("cargo:rustc-env=TARGET_ID={}", sdk_info.target_id);
        println!("cargo:warning=TARGET_ID is {}", sdk_info.target_id);
        println!("cargo:rustc-env=TARGET_NAME={}", sdk_info.target_name);
        println!("cargo:warning=TARGET_NAME is {}", sdk_info.target_name);
        println!("cargo:rustc-env=C_SDK_NAME={}", sdk_info.c_sdk_name);
        println!("cargo:warning=C_SDK_NAME is {}", sdk_info.c_sdk_name);
        println!("cargo:rustc-env=C_SDK_HASH={}", sdk_info.c_sdk_hash);
        println!("cargo:warning=C_SDK_HASH is {}", sdk_info.c_sdk_hash);
        println!("cargo:rustc-env=C_SDK_VERSION={}", sdk_info.c_sdk_version);
        println!("cargo:warning=C_SDK_VERSION is {}", sdk_info.c_sdk_version);
        Ok(())
    }

    fn cxdefines(&mut self) -> Result<(), SDKBuildError> {
        let mut makefile = File::open(self.device.c_sdk.join("Makefile.conf.cx"))
            .expect("Could not find Makefile.conf.cx");
        let mut content = String::new();
        makefile.read_to_string(&mut content).unwrap();
        // Extract the defines from the Makefile.conf.cx.
        // They all begin with `HAVE` and are ' ' and '\n' separated.
        let mut cxdefines = content
            .split('\n')
            .filter(|line| !line.starts_with('#')) // Remove lines that are commented
            .flat_map(|line| line.split(' ').filter(|word| word.starts_with("HAVE")))
            .map(|line| line.to_string())
            .collect::<Vec<String>>();

        cxdefines.push("NATIVE_LITTLE_ENDIAN".to_string());
        self.cxdefines = cxdefines;
        Ok(())
    }

    pub fn build_c_sdk(&self) -> Result<(), SDKBuildError> {
        // Generate glyphs
        generate_glyphs(&self.device);

        let mut command = cc::Build::new();
        if env::var_os("CC").is_none() {
            command.compiler("clang");
        } else {
            // Let cc::Build determine CC from the environment variable
        }

        command
            .files(&AUX_C_FILES)
            .files(str2path(&self.device.c_sdk, &SDK_C_FILES));

        let glyphs_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("glyphs");

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

        // Add the defines found in the Makefile.conf.cx to our build command.
        for define in self.cxdefines.iter() {
            command.define(define, None);
        }

        // Add defines and flags specified in the LEDGER_SDK_EXTRA_DEFINES and LEDGER_SDK_EXTRA_CFLAGS environment
        // variables, if they are set.
        // This allows apps to customize the build process. Since they are added after the default includes, they can
        // override previous definitions.

        println!("cargo:rerun-if-env-changed=LEDGER_SDK_EXTRA_DEFINES");
        println!("cargo:rerun-if-env-changed=LEDGER_SDK_EXTRA_CFLAGS");

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
        Ok(())
    }

    fn generate_bindings(&self) -> Result<(), SDKBuildError> {
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
                "include/os_pki.h", /* pki */
            ],
        );

        let mut bindings = bindgen::builder()
            .clang_args(&args)
            .prepend_enum_name(false)
            .generate_comments(false)
            .derive_default(true)
            .use_core();

        // Target specific files
        let csdk_target_name = self.device.name.to_string();
        let header = match self.device.name {
            DeviceName::NanoSPlus => {
                String::from("devices/nanosplus/c_sdk_build_nanosplus.defines")
            }
            DeviceName::NanoX => String::from("devices/nanox/c_sdk_build_nanox.defines"),
            DeviceName::Stax => String::from("devices/stax/c_sdk_build_stax.defines"),
            DeviceName::Flex => String::from("devices/flex/c_sdk_build_flex.defines"),
            DeviceName::ApexP => String::from("devices/apex_p/c_sdk_build_apex_p.defines"),
        };

        bindings = bindings.clang_arg(format!("-I{bsdk}/target/{csdk_target_name}/include/"));
        bindings = bindings.header(header);

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
        if ((self.device.name == DeviceName::NanoX || self.device.name == DeviceName::NanoSPlus)
            && env::var_os("CARGO_FEATURE_NANO_NBGL").is_some())
            || self.device.name == DeviceName::Stax
            || self.device.name == DeviceName::Flex
            || self.device.name == DeviceName::ApexP
        {
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
            if self.device.name == DeviceName::NanoSPlus || self.device.name == DeviceName::NanoX {
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

        let bindings = bindings
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            .generate()
            .expect("Unable to generate bindings");

        // Write the bindings to the $OUT_DIR/bindings.rs file.
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings");

        Ok(())
    }

    fn generate_heap_size(&self) -> Result<(), SDKBuildError> {
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
                    if let Some((k, v_str)) = entry.split_once(':') {
                        if k.trim() == target_os {
                            if let Ok(v) = v_str.trim().parse::<u32>() {
                                selected = Some(v);
                                break;
                            }
                        }
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
            "Invalid heap size specification '{raw}'; resolved value {heap_size_value} must be in [2048, {}] for target {}", max_heap_size, target_os
        );

        let out_dir = env::var("OUT_DIR").unwrap();
        let dest_path = Path::new(&out_dir).join("heap_size.rs");
        fs::write(
            &dest_path,
            format!("pub const HEAP_SIZE: usize = {heap_size_value};"),
        )
        .expect("Unable to write file");
        Ok(())
    }

    fn copy_linker_script(&self) -> Result<(), SDKBuildError> {
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
        Ok(())
    }
}

fn main() {
    let start = Instant::now();
    let mut sdk_builder = SDKBuilder::new();
    sdk_builder.gcc_toolchain().unwrap();
    sdk_builder.device().unwrap();
    sdk_builder.get_info().unwrap();
    sdk_builder.cxdefines().unwrap();
    sdk_builder.build_c_sdk().unwrap();
    sdk_builder.generate_bindings().unwrap();
    sdk_builder.generate_heap_size().unwrap();
    sdk_builder.copy_linker_script().unwrap();
    let end = start.elapsed();
    println!(
        "cargo:warning=Total build.rs time: {} seconds",
        end.as_secs()
    );
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
    println!("cargo:rustc-env=C_SDK_GRAPHICS={}", "nbgl");

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

fn retrieve_csdk_info(device: &Device, path: &PathBuf) -> Result<CSDKInfo, SDKBuildError> {
    let mut csdk_info = CSDKInfo::new();
    (csdk_info.api_level, csdk_info.c_sdk_name) = retrieve_makefile_infos(path)?;
    (csdk_info.target_id, csdk_info.target_name) = retrieve_target_file_infos(device, path)?;
    (csdk_info.c_sdk_hash, csdk_info.c_sdk_version) = retrieve_csdk_git_info(path);
    Ok(csdk_info)
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

fn retrieve_makefile_infos(c_sdk: &Path) -> Result<(Option<u32>, String), SDKBuildError> {
    let makefile =
        File::open(c_sdk.join("Makefile.defines")).expect("Could not find Makefile.defines");
    let mut api_level: Option<u32> = None;
    for line in BufReader::new(makefile).lines().flatten() {
        if let Some(value) = line.split(":=").nth(1).map(str::trim) {
            if line.contains("API_LEVEL") && api_level.is_none() {
                api_level = Some(value.parse().map_err(|_| SDKBuildError::InvalidAPILevel)?);
            }
        }
        if api_level.is_some() {
            // Key found, break out of the loop
            break;
        }
    }
    let makefile =
        File::open(c_sdk.join("Makefile.target")).expect("Could not find Makefile.defines");
    let mut sdk_name: Option<String> = None;
    for line in BufReader::new(makefile).lines().flatten() {
        if let Some(value) = line.split(":=").nth(1).map(str::trim) {
            if line.contains("SDK_NAME") && sdk_name.is_none() {
                sdk_name = Some(value.to_string().replace('\"', ""));
            }
        }
        if sdk_name.is_some() {
            // Key found, break out of the loop
            break;
        }
    }

    let sdk_name = sdk_name.ok_or(SDKBuildError::MissingSDKName)?;
    Ok((api_level, sdk_name))
}

fn retrieve_target_file_infos(
    device: &Device,
    c_sdk: &Path,
) -> Result<(String, String), SDKBuildError> {
    let prefix = format!("target/{}/", device.name);
    let target_file_path = c_sdk.join(format!("{}include/bolos_target.h", prefix));
    let target_file =
        File::open(target_file_path).map_err(|_| SDKBuildError::TargetFileNotFound)?;
    let mut target_id: Option<String> = None;
    let mut target_name: Option<String> = None;

    for line in BufReader::new(target_file).lines().flatten() {
        if target_id.is_none() && line.contains("#define TARGET_ID") {
            target_id = Some(
                line.split_whitespace()
                    .nth(2)
                    .ok_or("err")
                    .map_err(|_| SDKBuildError::MissingTargetId)?
                    .to_string(),
            );
        } else if target_name.is_none()
            && line.contains("#define TARGET_")
            && !line.contains("#define TARGET_ID")
        {
            target_name = Some(
                line.split_whitespace()
                    .nth(1)
                    .ok_or("err")
                    .map_err(|_| SDKBuildError::MissingTargetName)?
                    .to_string(),
            );
        }

        if target_id.is_some() && target_name.is_some() {
            // Both tokens found, break out of the loop
            break;
        }
    }

    let target_id = target_id.ok_or(SDKBuildError::MissingTargetId)?;
    let target_name = target_name.ok_or(SDKBuildError::MissingTargetName)?;
    Ok((target_id, target_name))
}

/// Fetch the appropriate C SDK to build
#[allow(dead_code)]
fn clone_sdk(devicename: &DeviceName) -> PathBuf {
    let (repo_url, sdk_branch) = match devicename {
        DeviceName::NanoX => (
            Path::new("https://github.com/LedgerHQ/ledger-secure-sdk"),
            "API_LEVEL_24",
        ),
        DeviceName::NanoSPlus => (
            Path::new("https://github.com/LedgerHQ/ledger-secure-sdk"),
            "API_LEVEL_24",
        ),
        DeviceName::Stax => (
            Path::new("https://github.com/LedgerHQ/ledger-secure-sdk"),
            "API_LEVEL_24",
        ),
        DeviceName::Flex => (
            Path::new("https://github.com/LedgerHQ/ledger-secure-sdk"),
            "API_LEVEL_24",
        ),
        DeviceName::ApexP => (
            Path::new("https://github.com/LedgerHQ/ledger-secure-sdk"),
            "API_LEVEL_25",
        ),
    };

    let out_dir = env::var("OUT_DIR").unwrap();
    let c_sdk = Path::new(out_dir.as_str()).join("ledger-secure-sdk");
    if !c_sdk.exists() {
        Command::new("git")
            .arg("clone")
            .arg(repo_url.to_str().unwrap())
            .arg("-b")
            .arg(sdk_branch)
            .arg(c_sdk.as_path())
            .output()
            .ok();
    }
    c_sdk
}

fn generate_glyphs(device: &Device) {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dest_path = out_path.join("glyphs");
    if !dest_path.exists() {
        fs::create_dir_all(&dest_path).ok();
    }

    // NBGL Glyphs
    if ((device.name == DeviceName::NanoSPlus || device.name == DeviceName::NanoX)
        && env::var_os("CARGO_FEATURE_NANO_NBGL").is_some())
        || device.name == DeviceName::Stax
        || device.name == DeviceName::Flex
        || device.name == DeviceName::ApexP
    {
        println!("cargo:warning=NBGL glyphs are generated");
        let icon2glyph = device.c_sdk.join("lib_nbgl/tools/icon2glyph.py");

        let mut cmd = Command::new(icon2glyph.as_os_str());
        cmd.arg("--glyphcheader")
            .arg(dest_path.join("glyphs.h").as_os_str())
            .arg("--glyphcfile")
            .arg(dest_path.join("glyphs.c").as_os_str());

        if device.name == DeviceName::NanoSPlus || device.name == DeviceName::NanoX {
            cmd.arg("--reverse");
        }

        for folder in device.glyphs_folders.iter() {
            for file in std::fs::read_dir(folder).unwrap() {
                let path = file.unwrap().path();
                let path_str = path.to_str().unwrap().to_string();
                cmd.arg(path_str);
            }
        }
        let _ = cmd.output();
    }
    // BAGL Glyphs
    else {
        println!("cargo:warning=BAGL glyphs are generated");
        let icon2glyph = device.c_sdk.join("icon3.py");

        let mut cmd1 = Command::new("python3");
        cmd1.arg(icon2glyph.as_os_str());
        cmd1.arg("--glyphcheader");
        let mut cmd2 = Command::new("python3");
        cmd2.arg(icon2glyph.as_os_str());
        cmd2.arg("--glyphcfile").arg("--factorize");

        for folder in device.glyphs_folders.iter() {
            for file in std::fs::read_dir(folder).unwrap() {
                let path = file.unwrap().path();
                let path_str = path.to_str().unwrap().to_string();
                cmd1.arg(&path_str);
                cmd2.arg(&path_str);
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
