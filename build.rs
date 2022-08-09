extern crate cc;
use std::path::PathBuf;
use std::process::Command;
use std::{env, error::Error, fs::File, io::Read};

fn finalize_nanos_configuration(command: &mut cc::Build, bolos_sdk: &String) -> String {
    command
        .target("thumbv6m-none-eabi")
        .define("ST31", None)
        .define("TARGET_NANOS", None)
        .define("TARGET_ID", Some("0x31100004"))
        .define("BAGL_HEIGHT", Some("32"))
        .define("BAGL_WIDTH", Some("128"))
        .file(format!("{bolos_sdk}/nanos/syscalls.c"))
        .file(format!("{bolos_sdk}/nanos/cx_stubs.S"))
        .file(format!(
            "{bolos_sdk}/nanos/lib_cxng/src/cx_exported_functions.c"
        ))
        .include(format!("{bolos_sdk}/nanos/"))
        .include(format!("{bolos_sdk}/nanos/lib_cxng/include"))
        .flag("-fropi");
    format!("{bolos_sdk}/nanos/Makefile.conf.cx")
}

fn finalize_nanox_configuration(command: &mut cc::Build, bolos_sdk: &String) -> String {
    command
        .target("thumbv6m-none-eabi")
        .define("ST33", None)
        .define("TARGET_NANOX", None)
        .define("TARGET_ID", Some("0x33000004"))
        .define("BAGL_HEIGHT", Some("64"))
        .define("BAGL_WIDTH", Some("128"))
        .define("HAVE_SEPROXYHAL_MCU", None)
        .define("HAVE_MCU_PROTECT", None)
        .define("HAVE_SE_BUTTON", None)
        .define("HAVE_SE_SCREEN", None)
        .define("HAVE_MCU_SERIAL_STORAGE", None)
        .define("HAVE_BLE", None)
        .define("HAVE_BLE_APDU", None)
        .file(format!("{bolos_sdk}/nanox/ledger_protocol.c"))
        .file(format!(
            "{bolos_sdk}/nanox/lib_blewbxx/core/auto/ble_gap_aci.c"
        ))
        .file(format!(
            "{bolos_sdk}/nanox/lib_blewbxx/core/auto/ble_gatt_aci.c"
        ))
        .file(format!(
            "{bolos_sdk}/nanox/lib_blewbxx/core/auto/ble_hal_aci.c"
        ))
        .file(format!(
            "{bolos_sdk}/nanox/lib_blewbxx/core/auto/ble_hci_le.c"
        ))
        .file(format!(
            "{bolos_sdk}/nanox/lib_blewbxx/core/template/osal.c"
        ))
        .file(format!(
            "{bolos_sdk}/nanox/lib_blewbxx_impl/src/ledger_ble.c"
        ))
        .include(format!("{bolos_sdk}/nanox/lib_blewbxx/include"))
        .include(format!("{bolos_sdk}/nanox/lib_blewbxx/core"))
        .include(format!("{bolos_sdk}/nanox/lib_blewbxx/core/auto"))
        .include(format!("{bolos_sdk}/nanox/lib_blewbxx/core/template"))
        .include(format!("{bolos_sdk}/nanox/lib_blewbxx_impl/include"))
        .file(format!("{bolos_sdk}/nanox/syscalls.c"))
        .file(format!("{bolos_sdk}/nanox/cx_stubs.S"))
        .file(format!(
            "{bolos_sdk}/nanox/lib_cxng/src/cx_exported_functions.c"
        ))
        .include(format!("{bolos_sdk}/nanox/"))
        .include(format!("{bolos_sdk}/nanox/lib_cxng/include"))
        .flag("-mno-movt")
        .flag("-ffixed-r9")
        .flag("-fropi")
        .flag("-frwpi");
    configure_lib_bagl(command, bolos_sdk);
    format!("{bolos_sdk}/nanox/Makefile.conf.cx")
}

fn finalize_nanosplus_configuration(command: &mut cc::Build, bolos_sdk: &String) -> String {
    command
        .target("thumbv8m.main-none-eabi")
        .define("ST33K1M5", None)
        .define("TARGET_NANOS2", None)
        .define("TARGET_ID", Some("0x33100004"))
        .define("BAGL_HEIGHT", Some("64"))
        .define("BAGL_WIDTH", Some("128"))
        .define("HAVE_SE_BUTTON", None)
        .define("HAVE_SE_SCREEN", None)
        .define("HAVE_MCU_SERIAL_STORAGE", None)
        .file(format!("{bolos_sdk}/nanosplus/syscalls.c"))
        .file(format!("{bolos_sdk}/nanosplus/cx_stubs.S"))
        .file(format!(
            "{bolos_sdk}/nanosplus/lib_cxng/src/cx_exported_functions.c"
        ))
        .include(format!("{bolos_sdk}/nanosplus/"))
        .include(format!("{bolos_sdk}/nanosplus/lib_cxng/include"))
        .flag("-fropi")
        .flag("-frwpi");
    configure_lib_bagl(command, bolos_sdk);
    format!("{bolos_sdk}/nanosplus/Makefile.conf.cx")
}

fn configure_lib_bagl(command: &mut cc::Build, bolos_sdk: &String) {
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
            .include(format!("{bolos_sdk}/lib_bagl/src/"))
            .file(format!("{bolos_sdk}/lib_bagl/src/bagl.c"))
            .file(format!("{bolos_sdk}/lib_bagl/src/bagl_fonts.c"))
            .file(format!("{bolos_sdk}/lib_bagl/src/bagl_glyphs.c"));
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let bolos_sdk = "./ledger-secure-sdk".to_string();

    let output = Command::new("arm-none-eabi-gcc")
        .arg("-print-sysroot")
        .output().ok();
    let sysroot = output
        .as_ref().and_then(|o|std::str::from_utf8(&o.stdout).ok())
        .unwrap_or("").trim();

    let gcc_toolchain = if sysroot.is_empty() {
        String::from("/usr/include/")
    } else {
        format!("{sysroot}/include")
    };

    let mut command = cc::Build::new();
    if env::var_os("CC").is_none() {
        command.compiler("clang");
    } else {
        // Let cc::Build determine CC from the environment variable
    }

    command
        .file("./src/c/src.c")
        .file("./src/c/sjlj.s")
        .file(format!("{bolos_sdk}/src/os_io_usb.c"))
        .file(format!("{bolos_sdk}/src/pic.c"))
        .file(format!("{bolos_sdk}/src/checks.c"))
        .file(format!("{bolos_sdk}/src/os.c"))
        .file(format!("{bolos_sdk}/src/svc_call.s"))
        .file(format!("{bolos_sdk}/src/svc_cx_call.s"))
        .file(format!("{bolos_sdk}/lib_stusb/usbd_conf.c"))
        .file(format!(
            "{bolos_sdk}/lib_stusb/STM32_USB_Device_Library/Core/Src/usbd_core.c"
        ))
        .file(format!(
            "{bolos_sdk}/lib_stusb/STM32_USB_Device_Library/Core/Src/usbd_ctlreq.c"
        ))
        .file(format!(
            "{bolos_sdk}/lib_stusb/STM32_USB_Device_Library/Core/Src/usbd_ioreq.c"
        ))
        .file(format!("{bolos_sdk}/lib_stusb_impl/usbd_impl.c"))
        .file(format!(
            "{bolos_sdk}/lib_stusb/STM32_USB_Device_Library/Class/HID/Src/usbd_hid.c"
        ))
        .define("HAVE_LOCAL_APDU_BUFFER", None)
        .define("IO_HID_EP_LENGTH", Some("64"))
        .define("USB_SEGMENT_SIZE", Some("64"))
        .define("OS_IO_SEPROXYHAL", None)
        .define("HAVE_IO_USB", None)
        .define("HAVE_L4_USBLIB", None)
        .define("HAVE_USB_APDU", None)
        .define("__IO", Some("volatile"))
        .define("IO_USB_MAX_ENDPOINTS", Some("6"))
        .define("IO_SEPROXYHAL_BUFFER_SIZE_B", Some("128"))
        .include(gcc_toolchain)
        .include(format!("{bolos_sdk}/include"))
        .include(format!("{bolos_sdk}/lib_stusb"))
        .include(format!("{bolos_sdk}/lib_stusb_impl"))
        .include(format!(
            "{bolos_sdk}/lib_stusb/STM32_USB_Device_Library/Core/Inc"
        ))
        .include(format!(
            "{bolos_sdk}/lib_stusb/STM32_USB_Device_Library/Class/HID/Inc"
        ))
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
        .flag("-Wno-unused-command-line-argument");

    #[cfg(feature = "ccid")]
    {
        command = command
            .file(format!(
                "{bolos_sdk}/lib_stusb/STM32_USB_Device_Library/Class/CCID/src/usbd_ccid_cmd.c"
            ))
            .file(format!(
                "{bolos_sdk}/lib_stusb/STM32_USB_Device_Library/Class/CCID/src/usbd_ccid_core.c"
            ))
            .file(format!(
                "{bolos_sdk}/lib_stusb/STM32_USB_Device_Library/Class/CCID/src/usbd_ccid_if.c"
            ))
            .file(format!(
                "{bolos_sdk}/lib_stusb/STM32_USB_Device_Library/Class/CCID/src/usbd_ccid_cmd.c"
            ))
            .file(format!(
                "{bolos_sdk}/lib_stusb/STM32_USB_Device_Library/Class/CCID/src/usbd_ccid_core.c"
            ))
            .file(format!(
                "{bolos_sdk}/lib_stusb/STM32_USB_Device_Library/Class/CCID/src/usbd_ccid_if.c"
            ))
            .file(format!(
                "{bolos_sdk}/lib_stusb/STM32_USB_Device_Library/Class/CCID/src/usbd_ccid_cmd.c"
            ))
            .file(format!(
                "{bolos_sdk}/lib_stusb/STM32_USB_Device_Library/Class/CCID/src/usbd_ccid_core.c"
            ))
            .file(format!(
                "{bolos_sdk}/lib_stusb/STM32_USB_Device_Library/Class/CCID/src/usbd_ccid_if.c"
            ))
            .define("HAVE_USB_CLASS_CCID", None)
            .define("HAVE_CCID", None)
            .include(format!(
                "{bolos_sdk}/lib_stusb/STM32_USB_Device_Library/Class/CCID/inc"
            ))
            .include(format!(
                "{bolos_sdk}/lib_stusb/STM32_USB_Device_Library/Class/CCID/inc"
            ))
            .include(format!(
                "{bolos_sdk}/lib_stusb/STM32_USB_Device_Library/Class/CCID/inc"
            ))
            .clone();
    }

    enum Device {
        NanoS,
        NanoSPlus,
        NanoX,
    }
    use Device::*;

    // determine device
    let device = match env::var_os("CARGO_CFG_TARGET_OS").unwrap().to_str().unwrap() {
        "nanos" => NanoS,
        "nanosplus" => NanoSPlus,
        "nanox" => NanoX,
        target_name => panic!(
            "invalid target `{target_name}`, expected one of `nanos`, `nanox`, `nanosplus`. Run with `-Z build-std=core --target=./<target name>.json`"
        ),
    };

    let cx_makefile = match device {
        NanoS => finalize_nanos_configuration(&mut command, &bolos_sdk),
        NanoX => finalize_nanox_configuration(&mut command, &bolos_sdk),
        NanoSPlus => finalize_nanosplus_configuration(&mut command, &bolos_sdk),
    };

    if env::var_os("CARGO_FEATURE_PENDING_REVIEW_SCREEN").is_some() {
        command.define("HAVE_PENDING_REVIEW_SCREEN", None);
    }

    // all 'finalize_...' functions also declare a new 'cfg' variable corresponding
    // to the name of the target (as #[cfg(target = "nanox")] does not work, for example)
    // this allows code to easily import things depending on the target

    let mut makefile = File::open(cx_makefile).unwrap();
    let mut content = String::new();
    makefile.read_to_string(&mut content).unwrap();
    // Extract the defines from the Makefile.conf.cx.
    // They all begin with `HAVE` and are ' ' and '\n' separated.
    let mut defines = content
        .split('\n')
        .filter(|line| !line.starts_with('#')) // Remove lines that are commented
        .flat_map(|line| line.split(' ').filter(|word| word.starts_with("HAVE")))
        .collect::<Vec<&str>>();

    // do not forget NATIVE_LITTLE_ENDIAN
    let s = String::from("NATIVE_LITTLE_ENDIAN");
    defines.push(s.as_str());

    // Add the defines found in the Makefile.conf.cx to our build command.
    for define in defines {
        // scott could use for_each
        command.define(define, None);
    }

    command.compile("rust-app");

    // Copy this crate's linker script into the working directory of
    // the application so that it can be used there for the layout.
    // Trick taken from https://docs.rust-embedded.org/embedonomicon/main.html
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // extend the library search path
    println!("cargo:rustc-link-search={}", out_dir.display());
    // copy
    let linkerscript = match device {
        NanoS => "nanos_layout.ld",
        NanoX => "nanox_layout.ld",
        NanoSPlus => "nanosplus_layout.ld",
    };
    std::fs::copy(linkerscript, out_dir.join(linkerscript))?;
    std::fs::copy("link.ld", out_dir.join("link.ld"))?;
    Ok(())
}
