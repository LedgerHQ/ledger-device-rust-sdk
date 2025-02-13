extern crate cc;
use glob::glob;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use std::{env, fs::File, io::BufRead, io::BufReader, io::Read};

const AUX_C_FILES: [&str; 2] = ["./src/c/src.c", "./src/c/sjlj.s"];

const SDK_C_FILES: [&str; 8] = [
    "src/os_io_usb.c",
    "src/pic.c",
    "src/checks.c",
    "src/cx_stubs.S",
    "src/os.c",
    "src/svc_call.s",
    "src/svc_cx_call.s",
    "src/syscalls.c",
];

const SDK_USB_FILES: [&str; 6] = [
    "lib_stusb/usbd_conf.c",
    "lib_stusb/STM32_USB_Device_Library/Core/Src/usbd_core.c",
    "lib_stusb/STM32_USB_Device_Library/Core/Src/usbd_ctlreq.c",
    "lib_stusb/STM32_USB_Device_Library/Core/Src/usbd_ioreq.c",
    "lib_stusb_impl/usbd_impl.c",
    "lib_stusb/STM32_USB_Device_Library/Class/HID/Src/usbd_hid.c",
];

const CFLAGS_NANOS: [&str; 11] = [
    "-Oz",
    "-fomit-frame-pointer",
    "-fno-common",
    "-fdata-sections",
    "-ffunction-sections",
    "-mthumb",
    "-fno-jump-tables",
    "-fshort-enums",
    "-mno-unaligned-access",
    "-fropi",
    "-Wno-unused-command-line-argument",
];

const CFLAGS_NANOSPLUS: [&str; 22] = [
    "-Oz",
    "-g0",
    "-fomit-frame-pointer",
    "-momit-leaf-frame-pointer",
    "-fno-common",
    "-mlittle-endian",
    "-std=gnu99",
    "-fdata-sections",
    "-ffunction-sections",
    "-funsigned-char",
    "-fshort-enums",
    "-mno-unaligned-access",
    "-fropi",
    "-fno-jump-tables",
    "-nostdlib",
    "-nodefaultlibs",
    "-frwpi",
    "--target=armv8m-none-eabi",
    "-mcpu=cortex-m35p+nodsp",
    "-mthumb",
    "-msoft-float",
    "-Wno-unused-command-line-argument",
];
const CFLAGS_STAX: [&str; 22] = CFLAGS_NANOSPLUS;
const CFLAGS_FLEX: [&str; 22] = CFLAGS_NANOSPLUS;
const CFLAGS_NANOX: [&str; 21] = [
    "-Oz",
    "-g0",
    "-fomit-frame-pointer",
    "-momit-leaf-frame-pointer",
    "-fno-common",
    "-mlittle-endian",
    "-std=gnu99",
    "-fdata-sections",
    "-ffunction-sections",
    "-funsigned-char",
    "-fshort-enums",
    "-mno-unaligned-access",
    "-fropi",
    "-fno-jump-tables",
    "-nostdlib",
    "-nodefaultlibs",
    "-frwpi",
    "-mthumb",
    "--target=armv6m-none-eabi",
    "-mcpu=cortex-m0plus",
    "-Wno-unused-command-line-argument",
];

#[derive(Debug, Default, PartialEq)]
enum DeviceName {
    NanoS,
    #[default]
    NanoSPlus,
    NanoX,
    Stax,
    Flex,
}

#[derive(Debug, Default)]
struct Device<'a> {
    pub name: DeviceName,
    pub defines: Vec<(String, Option<String>)>,
    pub cflags: Vec<&'a str>,
}

impl std::fmt::Display for DeviceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceName::NanoS => write!(f, "nanos"),
            DeviceName::NanoSPlus => write!(f, "nanos2"),
            DeviceName::NanoX => write!(f, "nanox"),
            DeviceName::Stax => write!(f, "stax"),
            DeviceName::Flex => write!(f, "flex"),
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

fn retrieve_csdk_info(device: &Device, path: &PathBuf) -> Result<CSDKInfo, SDKBuildError> {
    let mut csdk_info = CSDKInfo::new();
    (csdk_info.api_level, csdk_info.c_sdk_name) = retrieve_makefile_infos(path)?;
    (csdk_info.target_id, csdk_info.target_name) = retrieve_target_file_infos(device, path)?;
    (csdk_info.c_sdk_hash, csdk_info.c_sdk_version) = retrieve_csdk_git_info(path);
    Ok(csdk_info)
}

fn retrieve_csdk_git_info(bolos_sdk: &Path) -> (String, String) {
    let c_sdk_hash = match Command::new("git")
        .arg("-C")
        .arg(bolos_sdk)
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
        .arg(bolos_sdk)
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

fn retrieve_makefile_infos(bolos_sdk: &Path) -> Result<(Option<u32>, String), SDKBuildError> {
    let makefile =
        File::open(bolos_sdk.join("Makefile.defines")).expect("Could not find Makefile.defines");
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
        File::open(bolos_sdk.join("Makefile.target")).expect("Could not find Makefile.defines");
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
    bolos_sdk: &Path,
) -> Result<(String, String), SDKBuildError> {
    let prefix = if device.name == DeviceName::NanoS {
        "".to_string()
    } else {
        format!("target/{}/", device.name)
    };
    let target_file_path = bolos_sdk.join(format!("{}include/bolos_target.h", prefix));
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
fn clone_sdk(device: &Device) -> PathBuf {
    let (repo_url, sdk_branch) = match device.name {
        DeviceName::NanoS => (
            Path::new("https://github.com/LedgerHQ/ledger-secure-sdk"),
            "API_LEVEL_LNS",
        ),
        DeviceName::NanoX => (
            Path::new("https://github.com/LedgerHQ/ledger-secure-sdk"),
            "API_LEVEL_22",
        ),
        DeviceName::NanoSPlus => (
            Path::new("https://github.com/LedgerHQ/ledger-secure-sdk"),
            "API_LEVEL_22",
        ),
        DeviceName::Stax => (
            Path::new("https://github.com/LedgerHQ/ledger-secure-sdk"),
            "API_LEVEL_22",
        ),
        DeviceName::Flex => (
            Path::new("https://github.com/LedgerHQ/ledger-secure-sdk"),
            "API_LEVEL_22",
        ),
    };

    let out_dir = env::var("OUT_DIR").unwrap();
    let bolos_sdk = Path::new(out_dir.as_str()).join("ledger-secure-sdk");
    if !bolos_sdk.exists() {
        Command::new("git")
            .arg("clone")
            .arg(repo_url.to_str().unwrap())
            .arg("-b")
            .arg(sdk_branch)
            .arg(bolos_sdk.as_path())
            .output()
            .ok();
    }
    bolos_sdk
}

#[derive(Debug)]
enum SDKBuildError {
    UnsupportedDevice,
    InvalidAPILevel,
    MissingSDKName,
    TargetFileNotFound,
    MissingTargetId,
    MissingTargetName,
}

/// Helper function to concatenate all paths in pathlist to bolos_sdk's path
fn str2path(bolos_sdk: &Path, pathlist: &[&str]) -> Vec<PathBuf> {
    pathlist
        .iter()
        .map(|p| bolos_sdk.join(p))
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

struct SDKBuilder<'a> {
    bolos_sdk: PathBuf,
    api_level: u32,
    gcc_toolchain: PathBuf,
    device: Device<'a>,
    glyphs_folders: Vec<PathBuf>,
    cxdefines: Vec<String>,
}

impl SDKBuilder<'_> {
    pub fn new() -> Self {
        SDKBuilder {
            bolos_sdk: PathBuf::new(),
            api_level: 0,
            gcc_toolchain: PathBuf::new(),
            device: Device::default(),
            glyphs_folders: Vec::new(),
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
        // determine device
        self.device = match env::var_os("CARGO_CFG_TARGET_OS")
            .unwrap()
            .to_str()
            .unwrap()
        {
            "nanos" => Device {
                name: DeviceName::NanoS,
                defines: header2define("sdk_nanos.h"),
                cflags: Vec::from(CFLAGS_NANOS),
            },
            "nanosplus" => Device {
                name: DeviceName::NanoSPlus,
                defines: header2define("sdk_nanos2.h"),
                cflags: Vec::from(CFLAGS_NANOSPLUS),
            },
            "nanox" => Device {
                name: DeviceName::NanoX,
                defines: header2define("sdk_nanox.h"),
                cflags: Vec::from(CFLAGS_NANOX),
            },
            "stax" => Device {
                name: DeviceName::Stax,
                defines: header2define("sdk_stax.h"),
                cflags: Vec::from(CFLAGS_STAX),
            },
            "flex" => Device {
                name: DeviceName::Flex,
                defines: header2define("sdk_flex.h"),
                cflags: Vec::from(CFLAGS_FLEX),
            },
            _ => {
                return Err(SDKBuildError::UnsupportedDevice);
            }
        };

        // export TARGET into env for 'infos.rs'
        println!("cargo:rustc-env=TARGET={}", self.device.name);
        println!("cargo:warning=Device is {:?}", self.device.name);
        Ok(())
    }

    pub fn bolos_sdk(&mut self) -> Result<(), SDKBuildError> {
        println!("cargo:rerun-if-env-changed=LEDGER_SDK_PATH");
        self.bolos_sdk = match env::var("LEDGER_SDK_PATH") {
            Err(_) => clone_sdk(&self.device),
            Ok(path) => PathBuf::from(path),
        };

        let sdk_info = retrieve_csdk_info(&self.device, &self.bolos_sdk)?;
        match sdk_info.api_level {
            Some(api_level) => {
                self.api_level = api_level;
                // Export api level into env for 'infos.rs'
                println!("cargo:rustc-env=API_LEVEL={}", self.api_level);
                println!("cargo:warning=API_LEVEL is {}", self.api_level);
            }
            None => {
                if self.device.name != DeviceName::NanoS {
                    return Err(SDKBuildError::InvalidAPILevel);
                }
            }
        }

        // set glyphs folders
        match self.device.name {
            DeviceName::Flex => {
                self.glyphs_folders
                    .push(self.bolos_sdk.join("lib_nbgl/glyphs/wallet"));
                self.glyphs_folders
                    .push(self.bolos_sdk.join("lib_nbgl/glyphs/64px"));
                self.glyphs_folders
                    .push(self.bolos_sdk.join("lib_nbgl/glyphs/40px"));
            }
            DeviceName::Stax => {
                self.glyphs_folders
                    .push(self.bolos_sdk.join("lib_nbgl/glyphs/wallet"));
                self.glyphs_folders
                    .push(self.bolos_sdk.join("lib_nbgl/glyphs/64px"));
                self.glyphs_folders
                    .push(self.bolos_sdk.join("lib_nbgl/glyphs/32px"));
            }
            _ => {
                self.glyphs_folders
                    .push(self.bolos_sdk.join("lib_nbgl/glyphs/nano"));
            }
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
        let mut makefile = File::open(self.bolos_sdk.join("Makefile.conf.cx"))
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

    pub fn generate_glyphs(&self) -> Result<(), SDKBuildError> {
        if self.device.name == DeviceName::NanoS {
            return Err(SDKBuildError::UnsupportedDevice);
        }

        let icon2glyph = self.bolos_sdk.join("lib_nbgl/tools/icon2glyph.py");

        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        let dest_path = out_path.join("glyphs");
        if !dest_path.exists() {
            fs::create_dir_all(&dest_path).ok();
        }

        let mut cmd = Command::new(icon2glyph.as_os_str());
        cmd.arg("--glyphcheader")
            .arg(dest_path.join("glyphs.h").as_os_str())
            .arg("--glyphcfile")
            .arg(dest_path.join("glyphs.c").as_os_str());

        for folder in self.glyphs_folders.iter() {
            for file in std::fs::read_dir(folder).unwrap() {
                let path = file.unwrap().path();
                let path_str = path.to_str().unwrap().to_string();
                cmd.arg(path_str);
            }
        }

        let _ = cmd.output();
        Ok(())
    }

    pub fn build_c_sdk(&self) -> Result<(), SDKBuildError> {
        let mut command = cc::Build::new();
        if env::var_os("CC").is_none() {
            command.compiler("clang");
        } else {
            // Let cc::Build determine CC from the environment variable
        }

        command
            .files(&AUX_C_FILES)
            .files(str2path(&self.bolos_sdk, &SDK_C_FILES))
            .files(str2path(&self.bolos_sdk, &SDK_USB_FILES));

        command = command
            .include(self.gcc_toolchain.join("include"))
            .include(self.bolos_sdk.join("include"))
            .include(self.bolos_sdk.join("lib_cxng/include"))
            .include(self.bolos_sdk.join("lib_stusb"))
            .include(self.bolos_sdk.join("lib_stusb_impl"))
            .include(
                self.bolos_sdk
                    .join("lib_stusb/STM32_USB_Device_Library/Core/Inc"),
            )
            .include(
                self.bolos_sdk
                    .join("lib_stusb/STM32_USB_Device_Library/Class/HID/Inc"),
            )
            .debug(true)
            .define("main", "_start")
            .clone();

        // Set the #defines
        for (define, value) in &self.device.defines {
            command.define(define.as_str(), value.as_deref());
        }

        // Set the CFLAGS
        for cflag in &self.device.cflags {
            command.flag(cflag);
        }

        match self.device.name {
            DeviceName::NanoS => finalize_nanos_configuration(&mut command, &self.bolos_sdk),
            DeviceName::NanoX => finalize_nanox_configuration(&mut command, &self.bolos_sdk),
            DeviceName::NanoSPlus => {
                finalize_nanosplus_configuration(&mut command, &self.bolos_sdk)
            }
            DeviceName::Stax => finalize_stax_configuration(&mut command, &self.bolos_sdk),
            DeviceName::Flex => finalize_flex_configuration(&mut command, &self.bolos_sdk),
        };

        // Add the defines found in the Makefile.conf.cx to our build command.
        for define in self.cxdefines.iter() {
            command.define(define, None);
        }

        command.compile("ledger-secure-sdk");

        /* Link with libc for unresolved symbols */
        let mut path = self.bolos_sdk.display().to_string();
        match self.device.name {
            DeviceName::NanoS => {
                path = self.gcc_toolchain.display().to_string();
                path.push_str("/lib");
            }
            DeviceName::NanoX => {
                path.push_str("/arch/st33/lib");
            }
            DeviceName::NanoSPlus | DeviceName::Flex | DeviceName::Stax => {
                path.push_str("/arch/st33k1/lib");
            }
        };
        println!("cargo:rustc-link-lib=c");
        println!("cargo:rustc-link-search={path}");
        Ok(())
    }

    fn generate_bindings(&self) -> Result<(), SDKBuildError> {
        let bsdk = self.bolos_sdk.display().to_string();
        let gcc_tc = self.gcc_toolchain.display().to_string();
        let args = [
            "--target=thumbv6m-none-eabi".to_string(), // exact target is irrelevant for bindings
            "-fshort-enums".to_string(),
            format!("-I{gcc_tc}/include"),
            format!("-I{bsdk}/include"),
            format!("-I{bsdk}/lib_cxng/include/"),
            format!("-I{bsdk}/lib_stusb/STM32_USB_Device_Library/Core/Inc/"),
            format!("-I{bsdk}/lib_stusb/"),
        ];
        let headers = str2path(
            &self.bolos_sdk,
            &[
                "lib_cxng/include/libcxng.h", /* cxlib */
                "include/os.h",               /* syscalls */
                "include/os_screen.h",
                "include/syscalls.h",
                "include/os_io_seproxyhal.h",
                "include/os_ux.h",
                "include/ox.h", /* crypto-related syscalls */
                "lib_stusb/STM32_USB_Device_Library/Core/Inc/usbd_def.h",
                "include/os_io_usb.h",
                "lib_standard_app/swap_lib_calls.h",
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
        let header = format!("sdk_{csdk_target_name}.h");

        bindings = bindings.clang_arg(format!("-I{bsdk}/target/{csdk_target_name}/include/"));
        bindings = bindings.header(header);

        // SDK headers to bind against
        for header in headers.iter().map(|p| p.to_str().unwrap()) {
            bindings = bindings.header(header);
        }

        // BAGL or NBGL bindings
        match self.device.name {
            DeviceName::NanoS => {
                bindings = bindings.header(self.bolos_sdk.join("include/bagl.h").to_str().unwrap())
            }
            DeviceName::NanoSPlus | DeviceName::NanoX | DeviceName::Stax | DeviceName::Flex => {
                if ((self.device.name == DeviceName::NanoX
                    || self.device.name == DeviceName::NanoSPlus)
                    && env::var_os("CARGO_FEATURE_NBGL").is_some())
                    || self.device.name == DeviceName::Stax
                    || self.device.name == DeviceName::Flex
                {
                    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
                    let mut include_path = "-I".to_string();
                    let glyphs = out_path.join("glyphs");
                    include_path += glyphs.to_str().unwrap();
                    bindings = bindings.clang_args([include_path.as_str()]);

                    bindings = bindings.clang_args([
                        format!("-I{bsdk}/lib_nbgl/include/").as_str(),
                        format!("-I{bsdk}/lib_ux_nbgl/").as_str(),
                    ]);
                    bindings = bindings
                        .header(
                            self.bolos_sdk
                                .join("lib_nbgl/include/nbgl_use_case.h")
                                .to_str()
                                .unwrap(),
                        )
                        .header(
                            self.bolos_sdk
                                .join("lib_ux_nbgl/ux_nbgl.h")
                                .to_str()
                                .unwrap(),
                        );
                }
            }
        }

        // BLE bindings
        match self.device.name {
            DeviceName::NanoX | DeviceName::Flex | DeviceName::Stax => {
                bindings = bindings.header(
                    self.bolos_sdk
                        .join("lib_blewbxx_impl/include/ledger_ble.h")
                        .to_str()
                        .unwrap(),
                )
            }
            _ => (),
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
        // Read the HEAP_SIZE environment variable, default to 8192 if not set
        let heap_size = env::var("HEAP_SIZE").unwrap_or_else(|_| "8192".to_string());

        let heap_size_value = heap_size.parse::<u32>().unwrap();

        assert!(
            heap_size_value >= 2048 && heap_size_value <= 24576,
            "Invalid heap size: {heap_size}; Shall be included in [2048, 24576]"
        );

        // Generate the heap_size.rs file with the HEAP_SIZE value
        let out_dir = env::var("OUT_DIR").unwrap();
        let dest_path = Path::new(&out_dir).join("heap_size.rs");
        fs::write(
            &dest_path,
            format!("pub const HEAP_SIZE: usize = {};", heap_size),
        )
        .expect("Unable to write file");
        Ok(())
    }
}

fn main() {
    let start = Instant::now();
    let mut sdk_builder = SDKBuilder::new();
    sdk_builder.gcc_toolchain().unwrap();
    sdk_builder.device().unwrap();
    sdk_builder.bolos_sdk().unwrap();
    sdk_builder.cxdefines().unwrap();
    sdk_builder.generate_glyphs().unwrap();
    sdk_builder.build_c_sdk().unwrap();
    sdk_builder.generate_bindings().unwrap();
    sdk_builder.generate_heap_size().unwrap();
    let end = start.elapsed();
    println!(
        "cargo:warning=Total build.rs time: {} seconds",
        end.as_secs()
    );
}

fn finalize_nanos_configuration(command: &mut cc::Build, bolos_sdk: &Path) {
    command
        .target("thumbv6m-none-eabi")
        .define("ST31", None)
        .include(bolos_sdk.join("target/nanos/include"));
}

fn finalize_nanox_configuration(command: &mut cc::Build, bolos_sdk: &Path) {
    command
        .target("thumbv6m-none-eabi")
        .include(bolos_sdk.join("target/nanox/include"));

    configure_lib_ble(command, bolos_sdk);

    if env::var_os("CARGO_FEATURE_NBGL").is_some() {
        println!("cargo:warning=NBGL is built");
        command.define("HAVE_NBGL", None);
        command.define("NBGL_STEP", None);
        command.define("NBGL_USE_CASE", None);
        configure_lib_nbgl(command, bolos_sdk);
    } else {
        println!("cargo:warning=BAGL is built");
        command.define("HAVE_BAGL", None);
    }
}

fn finalize_nanosplus_configuration(command: &mut cc::Build, bolos_sdk: &Path) {
    command
        .target("thumbv8m.main-none-eabi")
        .include(bolos_sdk.join("target/nanos2/include"));

    if env::var_os("CARGO_FEATURE_NBGL").is_some() {
        println!("cargo:warning=NBGL is built");
        command.define("HAVE_NBGL", None);
        command.define("NBGL_STEP", None);
        command.define("NBGL_USE_CASE", None);
        configure_lib_nbgl(command, bolos_sdk);
    } else {
        println!("cargo:warning=BAGL is built");
        command.define("HAVE_BAGL", None);
    }
}

fn finalize_stax_configuration(command: &mut cc::Build, bolos_sdk: &Path) {
    command
        .target("thumbv8m.main-none-eabi")
        .include(bolos_sdk.join("target/stax/include/"));

    configure_lib_ble(command, bolos_sdk);
    configure_lib_nbgl(command, bolos_sdk);
}

fn finalize_flex_configuration(command: &mut cc::Build, bolos_sdk: &Path) {
    command
        .target("thumbv8m.main-none-eabi")
        .include(bolos_sdk.join("target/flex/include/"));

    configure_lib_ble(command, bolos_sdk);
    configure_lib_nbgl(command, bolos_sdk);
}

fn configure_lib_ble(command: &mut cc::Build, bolos_sdk: &Path) {
    command
        .file(bolos_sdk.join("src/ledger_protocol.c"))
        .file(bolos_sdk.join("lib_blewbxx/core/auto/ble_gap_aci.c"))
        .file(bolos_sdk.join("lib_blewbxx/core/auto/ble_gatt_aci.c"))
        .file(bolos_sdk.join("lib_blewbxx/core/auto/ble_hal_aci.c"))
        .file(bolos_sdk.join("lib_blewbxx/core/auto/ble_hci_le.c"))
        .file(bolos_sdk.join("lib_blewbxx/core/auto/ble_l2cap_aci.c"))
        .file(bolos_sdk.join("lib_blewbxx/core/template/osal.c"))
        .file(bolos_sdk.join("lib_blewbxx_impl/src/ledger_ble.c"))
        .include(bolos_sdk.join("lib_blewbxx/include"))
        .include(bolos_sdk.join("lib_blewbxx/core"))
        .include(bolos_sdk.join("lib_blewbxx/core/auto"))
        .include(bolos_sdk.join("lib_blewbxx/core/template"))
        .include(bolos_sdk.join("lib_blewbxx_impl/include"));
}

fn configure_lib_nbgl(command: &mut cc::Build, bolos_sdk: &Path) {
    let glyphs_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("glyphs");
    command
        .include(bolos_sdk.join("lib_nbgl/include/"))
        .include(bolos_sdk.join("lib_nbgl/include/fonts/"))
        .include(bolos_sdk.join("lib_ux_nbgl/"))
        .include(bolos_sdk.join("qrcode/include/"))
        .include(bolos_sdk.join("lib_bagl/include/"))
        .file(bolos_sdk.join("lib_ux_nbgl/ux.c"))
        .file(bolos_sdk.join("lib_bagl/src/bagl_fonts.c"))
        .file(bolos_sdk.join("src/os_printf.c"))
        .file(bolos_sdk.join("qrcode/src/qrcodegen.c"))
        .files(
            glob(bolos_sdk.join("lib_nbgl/src/*.c").to_str().unwrap())
                .unwrap()
                .map(|x| x.unwrap())
                .collect::<Vec<PathBuf>>(),
        )
        .include(&glyphs_path)
        .file(glyphs_path.join("glyphs.c"));
}
