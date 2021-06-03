extern crate cc;
use std::path::PathBuf;
use std::process::Command;
use std::{
    env,
    error::Error,
    fs::File,
    io::{Read, Write},
};

fn main() -> Result<(), Box<dyn Error>> {
    let bolos_sdk = "./nanos-secure-sdk".to_string();

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
        .target("thumbv6m-none-eabi")
        .file("./src/c/src.c")
        .file("./src/c/sjlj.s")
        .file(format!("{}/src/os_io_usb.c", bolos_sdk))
        .file(format!("{}/src/pic_internal.c", bolos_sdk))
        .file(format!("{}/src/pic.c", bolos_sdk))
        .file(format!("{}/src/svc_call.s", bolos_sdk))
        .file(format!("{}/src/svc_cx_call.s", bolos_sdk))
        .file(format!("{}/src/syscalls.c", bolos_sdk))
        .file(format!("{}/src/cx_stubs.S", bolos_sdk))
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
        .file(format!(
            "{}/lib_cxng/src/cx_exported_functions.c",
            bolos_sdk
        ))
        // The following flags should be the same as in wrapper
        //TODO : try to get rid of the flags in wrapper.h by using
        //      bindgen from within build.rs
        .define("ST31", None)
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
        .include(format!("{}/lib_cxng/include", bolos_sdk))
        .include(format!(
            "{}/lib_stusb/STM32_USB_Device_Library/Core/Inc",
            bolos_sdk
        ))
        .include(format!(
            "{}/lib_stusb/STM32_USB_Device_Library/Class/HID/Inc",
            bolos_sdk
        ))
        // More or less same flags as in the
        // C SDK Makefile.defines
        .no_default_flags(true)
        .pic(true)
        .flag("-fropi")
        .flag("--target=thumbv6m-none-eabi")
        .flag("-fomit-frame-pointer")
        .flag("-mcpu=cortex-m0")
        .flag("-fno-common")
        .flag("-fdata-sections")
        .flag("-ffunction-sections")
        .flag("-mtune=cortex-m0")
        .flag("-mthumb")
        .flag("-fno-jump-tables")
        .flag("-fno-builtin")
        .flag("-fshort-enums")
        .flag("-mno-unaligned-access")
        .flag("-Wno-unused-command-line-argument")
        .flag("-Wno-missing-declarations")
        .flag("-Wno-unused-parameter")
        .flag("-Wno-implicit-fallthrough")
        .flag("-Wno-sign-compare")
        .flag("-Wno-unknown-pragmas")
        .flag("-Wno-unknown-attributes")
        .flag("-Wno-pointer-sign")
        .flag("-Wno-implicit-function-declaration")
        .flag("-Wno-tautological-pointer-compare")
        .flag("-Wno-incompatible-pointer-types-discards-qualifiers")
        .flag("-Wno-duplicate-decl-specifier")
        .flag("-Wno-#warnings")
        .flag("-Wno-int-conversion")
        .clone();

    let mut makefile = File::open(format!("{}/Makefile.conf.cx", bolos_sdk)).unwrap();
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
    File::create(out_dir.join("script.ld"))?.write_all(include_bytes!("script.ld"))?;

    Ok(())
}
