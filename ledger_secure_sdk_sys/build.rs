extern crate cc;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs::File, io::BufRead, io::BufReader, io::Read};

// Definitions common to both `cc` and `bindgen`
const DEFINES: [(&str, Option<&str>); 11] = [
    ("HAVE_LOCAL_APDU_BUFFER", None),
    ("IO_HID_EP_LENGTH", Some("64")),
    ("USB_SEGMENT_SIZE", Some("64")),
    ("OS_IO_SEPROXYHAL", None),
    ("HAVE_IO_USB", None),
    ("HAVE_L4_USBLIB", None),
    ("HAVE_USB_APDU", None),
    ("__IO", Some("volatile")),
    ("IO_USB_MAX_ENDPOINTS", Some("6")),
    ("IO_SEPROXYHAL_BUFFER_SIZE_B", Some("128")),
    ("main", Some("_start")),
];

// Feature-specific definitions
const DEFINES_BLE: [(&str, Option<&str>); 2] = [("HAVE_BLE", None), ("HAVE_BLE_APDU", None)];

#[cfg(feature = "ccid")]
const DEFINES_CCID: [(&str, Option<&str>); 2] =
    [("HAVE_USB_CLASS_CCID", None), ("HAVE_CCID", None)];

const DEFINES_OPTIONAL: [(&str, Option<&str>); 7] = [
    ("HAVE_SEPROXYHAL_MCU", None),
    ("HAVE_MCU_PROTECT", None),
    ("HAVE_MCU_SEPROXYHAL", None),
    ("HAVE_MCU_SERIAL_STORAGE", None),
    ("HAVE_SE_BUTTON", None),
    ("HAVE_BAGL", None),
    ("HAVE_SE_SCREEN", None),
];

const AUX_C_FILES: [&str; 2] = ["./src/c/src.c", "./src/c/sjlj.s"];

const SDK_C_FILES: [&str; 9] = [
    "src/os_io_usb.c",
    "src/pic.c",
    "src/checks.c",
    "lib_cxng/src/cx_exported_functions.c",
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

#[derive(Debug)]
enum Device {
    NanoS,
    NanoSPlus,
    NanoX,
}

/// Fetch the appropriate C SDK to build
fn clone_sdk(device: &Device) -> (PathBuf, u32) {
    let (repo_url, sdk_branch, api_level) = match device {
        Device::NanoS => ("https://github.com/LedgerHQ/nanos-secure-sdk", "master", 0),
        Device::NanoX => (
            "https://github.com/LedgerHQ/ledger-secure-sdk",
            "API_LEVEL_5",
            5,
        ),
        Device::NanoSPlus => (
            "https://github.com/LedgerHQ/ledger-secure-sdk",
            "API_LEVEL_1",
            1,
        ),
    };

    let out_dir = env::var("OUT_DIR").unwrap();
    let bolos_sdk = Path::new(out_dir.as_str()).join("ledger-secure-sdk");
    if !bolos_sdk.exists() {
        Command::new("git")
            .arg("clone")
            .arg(repo_url)
            .arg("-b")
            .arg(sdk_branch)
            .arg(bolos_sdk.as_path())
            .output()
            .ok();
    }
    (bolos_sdk, api_level)
}

#[derive(Debug)]
enum SDKBuildError {
    InvalidAPILevel,
    CouldNotGetAPILevel,
}

/// Helper function to concatenate all paths in pathlist to bolos_sdk's path
fn str2path(bolos_sdk: &Path, pathlist: &[&str]) -> Vec<PathBuf> {
    pathlist
        .iter()
        .map(|p| bolos_sdk.join(p))
        .collect::<Vec<PathBuf>>()
}

struct SDKBuilder {
    bolos_sdk: PathBuf,
    api_level: u32,
    gcc_toolchain: String,
    device: Device,
    cxdefines: Vec<String>,
}

impl SDKBuilder {
    pub fn new() -> Self {
        SDKBuilder {
            bolos_sdk: PathBuf::new(),
            api_level: 0,
            gcc_toolchain: "".to_string(),
            device: Device::NanoS,
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
            String::from("/usr/include/")
        } else {
            format!("{sysroot}/include")
        };
        self.gcc_toolchain = gcc_toolchain;
    }

    pub fn device(&mut self) {
        // determine device
        let device = match env::var_os("CARGO_CFG_TARGET_OS").unwrap().to_str().unwrap() {
            "nanos" => Device::NanoS,
            "nanosplus" => Device::NanoSPlus,
            "nanox" => Device::NanoX,
            target_name => panic!(
                "invalid target `{target_name}`, expected one of `nanos`, `nanox`, `nanosplus`. Run with `-Z build-std=core --target=./<target name>.json`"
            ),
        };
        self.device = device;
        println!("cargo:warning=Device is {:?}", self.device);
    }

    /// Manually retrieve API_LEVEL from an SDK in the case of
    /// path given through the LEDGER_SDK_PATH env variable
    fn retrieve_api_level(bolos_sdk: &Path) -> Result<u32, SDKBuildError> {
        let makefile_defines = File::open(bolos_sdk.join("Makefile.defines"))
            .expect("Could not find Makefile.defines");
        for line in BufReader::new(makefile_defines).lines().flatten() {
            if line.contains("API_LEVEL") {
                return line.split(":=").collect::<Vec<&str>>()[1]
                    .trim()
                    .parse()
                    .map_err(|_| SDKBuildError::InvalidAPILevel);
            }
        }
        Err(SDKBuildError::CouldNotGetAPILevel)
    }

    pub fn bolos_sdk(&mut self) -> Result<(), SDKBuildError> {
        println!("cargo:rerun-if-env-changed=LEDGER_SDK_PATH");
        let (bolos_sdk, api_level) = match env::var("LEDGER_SDK_PATH") {
            Err(_) => clone_sdk(&self.device),
            Ok(path) => {
                let sdkpath = Path::new(&path).to_path_buf();
                let apilevel = SDKBuilder::retrieve_api_level(&sdkpath)?;
                (sdkpath, apilevel)
            }
        };
        self.bolos_sdk = bolos_sdk;
        self.api_level = api_level;

        // export API_LEVEL into env for 'infos.rs'
        println!("cargo:rustc-env=API_LEVEL={}", self.api_level);
        println!("cargo:warning=API_LEVEL is {}", self.api_level);

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

        for (define, value) in DEFINES {
            command.define(define, value);
        }

        command = command
            .include(&self.gcc_toolchain)
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
        };

        if env::var_os("CARGO_FEATURE_PENDING_REVIEW_SCREEN").is_some() {
            command.define("HAVE_PENDING_REVIEW_SCREEN", None);
        }
        // Add the defines found in the Makefile.conf.cx to our build command.
        for define in self.cxdefines.iter() {
            command.define(define, None);
        }

        command.compile("ledger-secure-sdk");
    }

    fn generate_bindings(&self) {
        let bsdk = self.bolos_sdk.display().to_string();
        let args = [
            "--target=thumbv6m-none-eabi".to_string(), // exact target is irrelevant for bindings
            "-fshort-enums".to_string(),
            format!("-I{}", self.gcc_toolchain),
            format!("-I{bsdk}/include"),
            format!("-I{bsdk}/lib_cxng/include/"),
            format!("-I{bsdk}/lib_stusb/STM32_USB_Device_Library/Core/Inc/"),
            format!("-I{bsdk}/lib_stusb/"),
        ];

        let headers = str2path(
            &self.bolos_sdk,
            &[
                "lib_cxng/include/libcxng.h",
                "include/os.h",
                "include/os_screen.h",
                "include/syscalls.h",
                "include/os_io_seproxyhal.h",
                "include/os_ux.h",
                "include/ox.h",
                "lib_stusb/STM32_USB_Device_Library/Core/Inc/usbd_def.h",
            ],
        );

        let mut bindings = bindgen::Builder::default()
            .clang_args(&args)
            .prepend_enum_name(false)
            .generate_comments(false)
            .derive_default(true)
            .use_core();

        for header in headers.iter().map(|p| p.to_str().unwrap()) {
            bindings = bindings.header(header);
        }
        bindings = bindings.header("sdk.h");

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
            _ => (),
        }
        for (define, value) in DEFINES.iter().chain(DEFINES_BLE.iter()) {
            let flag = match value {
                Some(v) => format!("-D{define}={v}"),
                _ => format!("-D{define}"),
            };
            bindings = bindings.clang_arg(flag);
        }

        // Add in target main include path
        let include_path = match self.device {
            Device::NanoS => "nanos",
            Device::NanoX => "nanox",
            Device::NanoSPlus => "nanos2",
        };
        bindings = bindings.clang_arg(format!("-I{bsdk}/target/{include_path}/include/"));

        // Add in optional definitions tied to a specific device
        match self.device {
            Device::NanoX | Device::NanoSPlus => {
                for (define, value) in DEFINES_OPTIONAL {
                    let flag = match value {
                        Some(v) => format!("-D{define}={v}"),
                        _ => format!("-D{define}"),
                    };
                    bindings = bindings.clang_arg(flag);
                }
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
}

fn main() {
    let mut sdk_builder = SDKBuilder::new();
    sdk_builder.gcc_toolchain();
    sdk_builder.device();
    sdk_builder.bolos_sdk().unwrap();
    sdk_builder.cxdefines();
    sdk_builder.build_c_sdk();
    sdk_builder.generate_bindings();
}

fn finalize_nanos_configuration(command: &mut cc::Build, bolos_sdk: &Path) {
    command
        .target("thumbv6m-none-eabi")
        .define("ST31", None)
        .define("BAGL_HEIGHT", Some("32"))
        .define("BAGL_WIDTH", Some("128"))
        .include(bolos_sdk.join("target/nanos/include"))
        .flag("-fropi");
}

fn finalize_nanox_configuration(command: &mut cc::Build, bolos_sdk: &Path) {
    for (define, value) in DEFINES_BLE {
        command.define(define, value);
    }
    for (define, value) in DEFINES_OPTIONAL {
        command.define(define, value);
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
    for (define, value) in DEFINES_OPTIONAL {
        command.define(define, value);
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
