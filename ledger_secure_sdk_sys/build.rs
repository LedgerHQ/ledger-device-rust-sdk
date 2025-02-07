extern crate cc;
use glob::glob;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs::File, io::BufRead, io::BufReader, io::Read};

#[cfg(feature = "ccid")]
const DEFINES_CCID: [(&str, Option<&str>); 2] =
    [("HAVE_USB_CLASS_CCID", None), ("HAVE_CCID", None)];

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

#[cfg(feature = "ccid")]
const CCID_FILES: [&str; 9] = [
    "lib_stusb/STM32_USB_Device_Library/Class/CCID/src/usbd_ccid_cmd.c",
    "lib_stusb/STM32_USB_Device_Library/Class/CCID/src/usbd_ccid_core.c",
    "lib_stusb/STM32_USB_Device_Library/Class/CCID/src/usbd_ccid_if.c",
    "lib_stusb/STM32_USB_Device_Library/Class/CCID/src/usbd_ccid_cmd.c",
    "lib_stusb/STM32_USB_Device_Library/Class/CCID/src/usbd_ccid_core.c",
    "lib_stusb/STM32_USB_Device_Library/Class/CCID/src/usbd_ccid_if.c",
    "lib_stusb/STM32_USB_Device_Library/Class/CCID/src/usbd_ccid_cmd.c",
    "lib_stusb/STM32_USB_Device_Library/Class/CCID/src/usbd_ccid_core.c",
    "lib_stusb/STM32_USB_Device_Library/Class/CCID/src/usbd_ccid_if.c",
];

#[derive(Debug, PartialEq)]
enum Device {
    NanoS,
    NanoSPlus,
    NanoX,
    Stax,
    Flex,
}

impl std::fmt::Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Device::NanoS => write!(f, "nanos"),
            Device::NanoSPlus => write!(f, "nanos2"),
            Device::NanoX => write!(f, "nanox"),
            Device::Stax => write!(f, "stax"),
            Device::Flex => write!(f, "flex"),
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
    let prefix = if *device == Device::NanoS {
        "".to_string()
    } else {
        format!("target/{}/", device)
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
    let (repo_url, sdk_branch) = match device {
        Device::NanoS => (
            Path::new("https://github.com/LedgerHQ/ledger-secure-sdk"),
            "API_LEVEL_LNS",
        ),
        Device::NanoX => (
            Path::new("https://github.com/LedgerHQ/ledger-secure-sdk"),
            "API_LEVEL_22",
        ),
        Device::NanoSPlus => (
            Path::new("https://github.com/LedgerHQ/ledger-secure-sdk"),
            "API_LEVEL_22",
        ),
        Device::Stax => (
            Path::new("https://github.com/LedgerHQ/ledger-secure-sdk"),
            "API_LEVEL_22",
        ),
        Device::Flex => (
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
    UnknownDevice,
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

struct SDKBuilder {
    bolos_sdk: PathBuf,
    api_level: u32,
    gcc_toolchain: PathBuf,
    device: Device,
    cxdefines: Vec<String>,
}

impl SDKBuilder {
    pub fn new() -> Self {
        SDKBuilder {
            bolos_sdk: PathBuf::new(),
            api_level: 0,
            gcc_toolchain: PathBuf::new(),
            device: Device::NanoS,
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
            "nanos" => Device::NanoS,
            "nanosplus" => Device::NanoSPlus,
            "nanox" => Device::NanoX,
            "stax" => Device::Stax,
            "flex" => Device::Flex,
            _ => {
                return Err(SDKBuildError::UnknownDevice);
            }
        };

        // export TARGET into env for 'infos.rs'
        println!("cargo:rustc-env=TARGET={}", self.device);
        println!("cargo:warning=Device is {:?}", self.device);
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
                if self.device != Device::NanoS {
                    return Err(SDKBuildError::InvalidAPILevel);
                }
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

    fn cxdefines(&mut self) {
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
    }

    pub fn generate_glyphs(&self) {
        if self.device == Device::NanoS {
            return;
        }

        let icon2glyph = self.bolos_sdk.join("lib_nbgl/tools/icon2glyph.py");

        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        let dest_path = match self.device {
            Device::Flex => out_path.join("glyphs_flex"),
            Device::Stax => out_path.join("glyphs_stax"),
            Device::NanoSPlus => out_path.join("glyphs_nanosplus"),
            Device::NanoX => out_path.join("glyphs_nanox"),
            Device::NanoS => panic!("Nano S does not support glyphs"),
        };
        if !dest_path.exists() {
            fs::create_dir_all(&dest_path).ok();
        }

        let mut glyph_folders: Vec<PathBuf> = Vec::new();
        match self.device {
            Device::Flex => {
                glyph_folders.push(self.bolos_sdk.join("lib_nbgl/glyphs/wallet"));
                glyph_folders.push(self.bolos_sdk.join("lib_nbgl/glyphs/64px"));
                glyph_folders.push(self.bolos_sdk.join("lib_nbgl/glyphs/40px"));
            }
            Device::Stax => {
                glyph_folders.push(self.bolos_sdk.join("lib_nbgl/glyphs/wallet"));
                glyph_folders.push(self.bolos_sdk.join("lib_nbgl/glyphs/64px"));
                glyph_folders.push(self.bolos_sdk.join("lib_nbgl/glyphs/32px"));
            }
            _ => {
                glyph_folders.push(self.bolos_sdk.join("lib_nbgl/glyphs/nano"));
            }
        }

        let mut cmd = Command::new(icon2glyph.as_os_str());
        cmd.arg("--glyphcheader")
            .arg(dest_path.join("glyphs.h").as_os_str())
            .arg("--glyphcfile")
            .arg(dest_path.join("glyphs.c").as_os_str());

        for folder in glyph_folders.iter() {
            for file in std::fs::read_dir(folder).unwrap() {
                let path = file.unwrap().path();
                let path_str = path.to_str().unwrap().to_string();
                cmd.arg(path_str);
            }
        }

        let _ = cmd.output();
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
            .flag("-Oz")
            .flag("-fomit-frame-pointer")
            .flag("-fno-common")
            .flag("-fdata-sections")
            .flag("-ffunction-sections")
            .flag("-mthumb")
            .flag("-fno-jump-tables")
            .flag("-fshort-enums")
            .flag("-mno-unaligned-access")
            .flag("-Wno-unused-command-line-argument")
            .clone();

        // #[cfg(feature = "ccid")]
        // {
        //     for (define, value) in DEFINES_CCID {
        //         command.define(define, value);
        //     }
        //     command.files(str2path(&self.bolos_sdk, &CCID_FILES));
        // }

        match self.device {
            Device::NanoS => finalize_nanos_configuration(&mut command, &self.bolos_sdk),
            Device::NanoX => finalize_nanox_configuration(&mut command, &self.bolos_sdk),
            Device::NanoSPlus => finalize_nanosplus_configuration(&mut command, &self.bolos_sdk),
            Device::Stax => finalize_stax_configuration(&mut command, &self.bolos_sdk),
            Device::Flex => finalize_flex_configuration(&mut command, &self.bolos_sdk),
        };

        // Add the defines found in the Makefile.conf.cx to our build command.
        for define in self.cxdefines.iter() {
            command.define(define, None);
        }

        command.compile("ledger-secure-sdk");

        /* Link with libc for unresolved symbols */
        let mut path = self.bolos_sdk.display().to_string();
        match self.device {
            Device::NanoS => {
                path = self.gcc_toolchain.display().to_string();
                path.push_str("/lib");
            }
            Device::NanoX => {
                path.push_str("/arch/st33/lib");
            }
            Device::NanoSPlus | Device::Flex | Device::Stax => {
                path.push_str("/arch/st33k1/lib");
            }
        };
        println!("cargo:rustc-link-lib=c");
        println!("cargo:rustc-link-search={path}");
    }

    fn generate_bindings(&self) {
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
        let (include_path, header) = match self.device {
            Device::NanoS => ("nanos", "sdk_nanos.h"),
            Device::NanoX => ("nanox", "sdk_nanox.h"),
            Device::NanoSPlus => ("nanos2", "sdk_nanosp.h"),
            Device::Stax => ("stax", "sdk_stax.h"),
            Device::Flex => ("flex", "sdk_flex.h"),
        };
        bindings = bindings.clang_arg(format!("-I{bsdk}/target/{include_path}/include/"));
        bindings = bindings.header(header);

        // SDK headers to bind against
        for header in headers.iter().map(|p| p.to_str().unwrap()) {
            bindings = bindings.header(header);
        }

        match self.device {
            Device::NanoS => {
                bindings = bindings.header(self.bolos_sdk.join("include/bagl.h").to_str().unwrap())
            }
            Device::NanoX => {
                bindings = bindings.header(
                    self.bolos_sdk
                        .join("lib_blewbxx_impl/include/ledger_ble.h")
                        .to_str()
                        .unwrap(),
                )
            }
            Device::Stax | Device::Flex => {
                let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
                let mut include_path = "-I".to_string();
                let glyphs = match self.device {
                    Device::Stax => out_path.join("glyphs_stax"),
                    Device::Flex => out_path.join("glyphs_flex"),
                    _ => panic!("Invalid device"),
                };
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
                    )
                    .header(
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
    }

    fn generate_heap_size(&self) {
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
    }
}

fn main() {
    let mut sdk_builder = SDKBuilder::new();
    sdk_builder.gcc_toolchain().unwrap();
    sdk_builder.device().unwrap();
    sdk_builder.bolos_sdk().unwrap();
    sdk_builder.cxdefines();
    sdk_builder.generate_glyphs();

    sdk_builder.build_c_sdk();
    sdk_builder.generate_bindings();
    sdk_builder.generate_heap_size();
}

fn finalize_nanos_configuration(command: &mut cc::Build, bolos_sdk: &Path) {
    let defines = header2define("sdk_nanos.h");
    for (define, value) in defines {
        command.define(define.as_str(), value.as_deref());
    }

    command
        .target("thumbv6m-none-eabi")
        .define("ST31", None)
        .define("BAGL_HEIGHT", Some("32"))
        .define("BAGL_WIDTH", Some("128"))
        .include(bolos_sdk.join("target/nanos/include"))
        .flag("-fropi");
}

fn finalize_nanox_configuration(command: &mut cc::Build, bolos_sdk: &Path) {
    let defines = header2define("sdk_nanox.h");
    for (define, value) in defines {
        command.define(define.as_str(), value.as_deref());
    }

    command
        .target("thumbv6m-none-eabi")
        .define("ST33", None)
        .define("BAGL_HEIGHT", Some("64"))
        .define("BAGL_WIDTH", Some("128"))
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
        .include(bolos_sdk.join("lib_blewbxx_impl/include"))
        .include(bolos_sdk.join("target/nanox/include"))
        .flag("-mno-movt")
        .flag("-ffixed-r9")
        .flag("-fropi")
        .flag("-frwpi");
    configure_lib_bagl(command, bolos_sdk);
}

fn finalize_nanosplus_configuration(command: &mut cc::Build, bolos_sdk: &Path) {
    let defines = header2define("sdk_nanosp.h");
    for (define, value) in defines {
        command.define(define.as_str(), value.as_deref());
    }

    command
        .target("thumbv8m.main-none-eabi")
        .define("ST33K1M5", None)
        .define("BAGL_HEIGHT", Some("64"))
        .define("BAGL_WIDTH", Some("128"))
        .include(bolos_sdk.join("target/nanos2/include"))
        .flag("-fropi")
        .flag("-frwpi");
    configure_lib_bagl(command, bolos_sdk);
}

fn configure_lib_bagl(command: &mut cc::Build, bolos_sdk: &Path) {
    if env::var_os("CARGO_FEATURE_LIB_BAGL").is_some() {
        command
            .define("HAVE_BAGL", None)
            // Just include all the fonts for now; we can shrink the X and S+ images later.
            .define("HAVE_BAGL_FONT_LUCIDA_CONSOLE_8PX", None)
            .define("HAVE_BAGL_FONT_OPEN_SANS_LIGHT_16_22PX", None)
            .define("HAVE_BAGL_FONT_OPEN_SANS_REGULAR_8_11PX", None)
            .define("HAVE_BAGL_FONT_OPEN_SANS_REGULAR_10_13PX", None)
            .define("HAVE_BAGL_FONT_OPEN_SANS_REGULAR_11_14PX", None)
            .define("HAVE_BAGL_FONT_OPEN_SANS_REGULAR_13_18PX", None)
            .define("HAVE_BAGL_FONT_OPEN_SANS_REGULAR_22_30PX", None)
            .define("HAVE_BAGL_FONT_OPEN_SANS_SEMIBOLD_8_11PX", None)
            .define("HAVE_BAGL_FONT_OPEN_SANS_EXTRABOLD_11PX", None)
            .define("HAVE_BAGL_FONT_OPEN_SANS_LIGHT_16PX", None)
            .define("HAVE_BAGL_FONT_OPEN_SANS_REGULAR_11PX", None)
            .define("HAVE_BAGL_FONT_OPEN_SANS_SEMIBOLD_10_13PX", None)
            .define("HAVE_BAGL_FONT_OPEN_SANS_SEMIBOLD_11_16PX", None)
            .define("HAVE_BAGL_FONT_OPEN_SANS_SEMIBOLD_13_18PX", None)
            .define("HAVE_BAGL_FONT_SYMBOLS_0", None)
            .define("HAVE_BAGL_FONT_SYMBOLS_1", None)
            .include(bolos_sdk.join("lib_bagl/src/"))
            .file(bolos_sdk.join("lib_bagl/src/bagl.c"))
            .file(bolos_sdk.join("lib_bagl/src/bagl_fonts.c"))
            .file(bolos_sdk.join("lib_bagl/src/bagl_glyphs.c"));
    }
}

fn finalize_stax_configuration(command: &mut cc::Build, bolos_sdk: &Path) {
    let defines = header2define("sdk_stax.h");
    for (define, value) in defines {
        command.define(define.as_str(), value.as_deref());
    }

    let glyphs_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("glyphs_stax");
    command
        .target("thumbv8m.main-none-eabi")
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
        .include(bolos_sdk.join("lib_blewbxx_impl/include"))
        .include(bolos_sdk.join("target/stax/include/"))
        .flag("-fropi")
        .flag("-frwpi")
        .include(&glyphs_path)
        .file(glyphs_path.join("glyphs.c"));
    configure_lib_nbgl(command, bolos_sdk);
}

fn finalize_flex_configuration(command: &mut cc::Build, bolos_sdk: &Path) {
    let defines = header2define("sdk_flex.h");
    for (define, value) in defines {
        command.define(define.as_str(), value.as_deref());
    }

    let glyphs_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("glyphs_flex");
    command
        .target("thumbv8m.main-none-eabi")
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
        .include(bolos_sdk.join("lib_blewbxx_impl/include"))
        .include(bolos_sdk.join("target/flex/include/"))
        .flag("-fropi")
        .flag("-frwpi")
        .include(&glyphs_path)
        .file(glyphs_path.join("glyphs.c"));
    configure_lib_nbgl(command, bolos_sdk);
}

fn configure_lib_nbgl(command: &mut cc::Build, bolos_sdk: &Path) {
    command
        .flag("-Wno-microsoft-anon-tag")
        .flag("-fms-extensions")
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
        );
}
