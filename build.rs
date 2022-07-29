extern crate cc;
use std::path::PathBuf;
use std::process::Command;
use std::{
    env,
    error::Error,
    fs::File,
    io::Read,
};

fn finalize_nanos_configuration(command: &mut cc::Build, bolos_sdk: &String) -> String {
    command.target("thumbv6m-none-eabi")
        .define("ST31", None)
        .define("TARGET_NANOS", None)
        .define("TARGET_ID", Some("0x31100004"))
        .define("BAGL_HEIGHT", Some("32"))
        .define("BAGL_WIDTH", Some("128"))
        .file(format!("{}/nanos/syscalls.c", bolos_sdk))
        .file(format!("{}/nanos/cx_stubs.S", bolos_sdk))
        .file(format!("{}/nanos/lib_cxng/src/cx_exported_functions.c", bolos_sdk))
        .include(format!("{}/nanos/", bolos_sdk))
        .include(format!("{}/nanos/lib_cxng/include", bolos_sdk))
        .flag("-fropi");
        format!("{}/nanos/Makefile.conf.cx", bolos_sdk)
}

fn finalize_nanox_configuration(command: &mut cc::Build, bolos_sdk: &String) -> String {
    command.target("thumbv6m-none-eabi")
        .define("ST33", None)
        .define("TARGET_NANOX", None)
        .define("TARGET_ID", Some("0x33000004"))
        .define("BAGL_HEIGHT", Some("64"))
        .define("BAGL_WIDTH", Some("128"))
        .define("HAVE_SEPROXYHAL_MCU", None)
        .define("HAVE_MCU_PROTECT", None)
        .define("HAVE_SE_BUTTON", None)
        .define("HAVE_SE_SCREEN", None)
        .file(format!("{}/nanox/syscalls.c", bolos_sdk))
        .file(format!("{}/nanox/cx_stubs.S", bolos_sdk))
        .file(format!("{}/nanox/lib_cxng/src/cx_exported_functions.c", bolos_sdk))
        .include(format!("{}/nanox/", bolos_sdk))
        .include(format!("{}/nanox/lib_cxng/include", bolos_sdk))
        .flag("-mno-movt")
        .flag("-ffixed-r9")
        .flag("-fropi")
        .flag("-frwpi");
        format!("{}/nanox/Makefile.conf.cx", bolos_sdk)
}

fn finalize_nanosplus_configuration(command: &mut cc::Build, bolos_sdk: &String) -> String {
    command.target("thumbv8m.main-none-eabi")
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
        .file(format!("{}/nanosplus/lib_cxng/src/cx_exported_functions.c", bolos_sdk))
        .include(format!("{}/nanosplus/", bolos_sdk))
        .include(format!("{}/nanosplus/lib_cxng/include", bolos_sdk))
        .flag("-fropi")
        .flag("-frwpi");
        format!("{}/nanosplus/Makefile.conf.cx", bolos_sdk)
}


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

    // determine target
    let target = env::var_os("TARGET");
    let cx_makefile = match target.clone().unwrap().to_str().unwrap() {
        "nanos" => finalize_nanos_configuration(&mut command, &bolos_sdk),
        "nanox" => finalize_nanox_configuration(&mut command, &bolos_sdk),
        "nanosplus" => finalize_nanosplus_configuration(&mut command, &bolos_sdk),
        _ => "".to_string() 
    };

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
    let linkerscript = match target.unwrap().to_str().unwrap() {
        "nanos" => "nanos_script.ld",
        "nanox" => "nanox_script.ld",
        "nanosplus" => "nanosplus_script.ld",
        _ => ""
    };
    std::fs::copy(linkerscript, out_dir.join(linkerscript))?;
    Ok(())
}
