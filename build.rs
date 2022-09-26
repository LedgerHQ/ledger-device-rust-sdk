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
        .file(format!("{}/nanos/syscalls.c", bolos_sdk))
        .file(format!("{}/nanos/cx_stubs.S", bolos_sdk))
        .file(format!(
            "{}/nanos/lib_cxng/src/cx_exported_functions.c",
            bolos_sdk
        ))
        .include(format!("{}/nanos/", bolos_sdk))
        .include(format!("{}/nanos/lib_cxng/include", bolos_sdk))
        .flag("-fropi");
    format!("{}/nanos/Makefile.conf.cx", bolos_sdk)
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
        .file(format!("{}/nanox/ledger_protocol.c", bolos_sdk))
        .file(format!(
            "{}/nanox/lib_blewbxx/core/auto/ble_gap_aci.c",
            bolos_sdk
        ))
        .file(format!(
            "{}/nanox/lib_blewbxx/core/auto/ble_gatt_aci.c",
            bolos_sdk
        ))
        .file(format!(
            "{}/nanox/lib_blewbxx/core/auto/ble_hal_aci.c",
            bolos_sdk
        ))
        .file(format!(
            "{}/nanox/lib_blewbxx/core/auto/ble_hci_le.c",
            bolos_sdk
        ))
        .file(format!(
            "{}/nanox/lib_blewbxx/core/template/osal.c",
            bolos_sdk
        ))
        .file(format!(
            "{}/nanox/lib_blewbxx_impl/src/ledger_ble.c",
            bolos_sdk
        ))
        .include(format!("{}/nanox/lib_blewbxx/include", bolos_sdk))
        .include(format!("{}/nanox/lib_blewbxx/core", bolos_sdk))
        .include(format!("{}/nanox/lib_blewbxx/core/auto", bolos_sdk))
        .include(format!("{}/nanox/lib_blewbxx/core/template", bolos_sdk))
        .include(format!("{}/nanox/lib_blewbxx_impl/include", bolos_sdk))
        .file(format!("{}/nanox/syscalls.c", bolos_sdk))
        .file(format!("{}/nanox/cx_stubs.S", bolos_sdk))
        .file(format!(
            "{}/nanox/lib_cxng/src/cx_exported_functions.c",
            bolos_sdk
        ))
        .include(format!("{}/nanox/", bolos_sdk))
        .include(format!("{}/nanox/lib_cxng/include", bolos_sdk))
        .flag("-mno-movt")
        .flag("-ffixed-r9")
        .flag("-fropi")
        .flag("-frwpi");
    configure_lib_bagl(command, bolos_sdk);
    format!("{}/nanox/Makefile.conf.cx", bolos_sdk)
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
        .file(format!("{}/nanosplus/syscalls.c", bolos_sdk))
        .file(format!("{}/nanosplus/cx_stubs.S", bolos_sdk))
        .file(format!(
            "{}/nanosplus/lib_cxng/src/cx_exported_functions.c",
            bolos_sdk
        ))
        .include(format!("{}/nanosplus/", bolos_sdk))
        .include(format!("{}/nanosplus/lib_cxng/include", bolos_sdk))
        .flag("-fropi")
        .flag("-frwpi");
    configure_lib_bagl(command, bolos_sdk);
    format!("{}/nanosplus/Makefile.conf.cx", bolos_sdk)
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
            .include(format!("{}/lib_bagl/src/", bolos_sdk))
            .file(format!("{}/lib_bagl/src/bagl.c", bolos_sdk))
            .file(format!("{}/lib_bagl/src/bagl_fonts.c", bolos_sdk))
            .file(format!("{}/lib_bagl/src/bagl_glyphs.c", bolos_sdk));
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let bolos_sdk = "./ledger-secure-sdk".to_string();

    let output = Command::new("arm-none-eabi-gcc")
        .arg("-print-sysroot")
        .output()
        .expect("failed");

    let sysroot = std::str::from_utf8(&output.stdout).unwrap().trim();
    let gcc_toolchain = if sysroot.is_empty() {
        String::from("/usr/include/")
    } else {
        format!("{}/include", sysroot)
    };

    let mut command = cc::Build::new()
        .compiler("clang")
        .file("./src/c/src.c")
        .file("./src/c/sjlj.s")
        .file(format!("{}/src/os_io_usb.c", bolos_sdk))
        .file(format!("{}/src/pic.c", bolos_sdk))
        .file(format!("{}/src/svc_call.s", bolos_sdk))
        .file(format!("{}/src/svc_cx_call.s", bolos_sdk))
        .file(format!("{}/lib_stusb/usbd_conf.c", bolos_sdk))
        .file(format!(
            "{}/lib_stusb/STM32_USB_Device_Library/Core/Src/usbd_core.c",
            bolos_sdk
        ))
        .file(format!(
            "{}/lib_stusb/STM32_USB_Device_Library/Core/Src/usbd_ctlreq.c",
            bolos_sdk
        ))
        .file(format!(
            "{}/lib_stusb/STM32_USB_Device_Library/Core/Src/usbd_ioreq.c",
            bolos_sdk
        ))
        .file(format!("{}/lib_stusb_impl/usbd_impl.c", bolos_sdk))
        .file(format!(
            "{}/lib_stusb/STM32_USB_Device_Library/Class/HID/Src/usbd_hid.c",
            bolos_sdk
        ))
        .define("HAVE_LOCAL_APDU_BUFFER", None)
        .define("IO_HID_EP_LENGTH", Some("64"))
        .define("USB_SEGMENT_SIZE", Some("64"))
        .define("OS_IO_SEPROXYHAL", None)
        .define("HAVE_IO_USB", None)
        .define("HAVE_L4_USBLIB", None)
        .define("HAVE_USB_APDU", None)
        .define("IO_USB_MAX_ENDPOINTS", Some("6"))
        .define("IO_SEPROXYHAL_BUFFER_SIZE_B", Some("128"))
        .include(gcc_toolchain)
        .include(format!("{}/include", bolos_sdk))
        .include(format!("{}/lib_stusb", bolos_sdk))
        .include(format!("{}/lib_stusb_impl", bolos_sdk))
        .include(format!(
            "{}/lib_stusb/STM32_USB_Device_Library/Core/Inc",
            bolos_sdk
        ))
        .include(format!(
            "{}/lib_stusb/STM32_USB_Device_Library/Class/HID/Inc",
            bolos_sdk
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
        .flag("-Wno-unused-command-line-argument")
        .clone();

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
            "invalid target `{}`, expected one of `nanos`, `nanox`, `nanosplus`. Run with `-Z build-std=core --target=./<target name>.json`",
            target_name
        ),
    };

    let cx_makefile = match device {
        NanoS => finalize_nanos_configuration(&mut command, &bolos_sdk),
        NanoX => finalize_nanox_configuration(&mut command, &bolos_sdk),
        NanoSPlus => finalize_nanosplus_configuration(&mut command, &bolos_sdk),
    };

    // all 'finalize_...' functions also declare a new 'cfg' variable corresponding
    // to the name of the target (as #[cfg(target = "nanox")] does not work, for example)
    // this allows code to easily import things depending on the target

    let mut makefile = File::open(cx_makefile).unwrap();
    let mut content = String::new();
    makefile.read_to_string(&mut content).unwrap();
    // Extract the defines from the Makefile.conf.cx.
    // They all begin with `HAVE` and are ' ' and '\n' separated.
    let defines = content
        .split('\n')
        .filter(|line| !line.starts_with('#')) // Remove lines that are commented
        .flat_map(|line| line.split(' ').filter(|word| word.starts_with("HAVE")))
        .collect::<Vec<&str>>();

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
