extern crate cc;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn main() {
    let bolos_sdk = "./nanos-secure-sdk/".to_string();

    let output = Command::new("arm-none-eabi-gcc")
        .arg("-print-sysroot")
        .output()
        .expect("failed");

    let sysroot = std::str::from_utf8(&output.stdout).unwrap().trim();
    let gcc_toolchain = if sysroot.is_empty() {
                            String::from("/usr/include/")
    } else {
        format!("{}/include",sysroot)
    };

    #[cfg(windows)]
    let py_cmd = "python";

    #[cfg(unix)]
    let py_cmd = "python3";

    let output = Command::new(py_cmd)
        .arg(&format!("./{}/icon3.py", bolos_sdk))
        .arg(&format!("{}/lib_ux/glyphs/icon_down.gif", bolos_sdk))
        .arg(&format!("{}/lib_ux/glyphs/icon_left.gif", bolos_sdk))
        .arg(&format!("{}/lib_ux/glyphs/icon_right.gif", bolos_sdk))
        .arg(&format!("{}/lib_ux/glyphs/icon_up.gif", bolos_sdk))
        .arg("--glyphcfile")
        .output()
        .expect("failed");

    let main_path = format!("{}/lib_ux/glyphs/", bolos_sdk);
    let dest_path = Path::new(&main_path);
    let mut f = File::create(&dest_path.join("glyphs.c")).unwrap();

    f.write_all(&output.stdout).unwrap();

    println!("{:?}", output.stderr);

    let output = Command::new(py_cmd)
        .arg(&format!("{}/icon3.py", bolos_sdk))
        .arg(&format!("{}/lib_ux/glyphs/icon_down.gif", bolos_sdk))
        .arg(&format!("{}/lib_ux/glyphs/icon_left.gif", bolos_sdk))
        .arg(&format!("{}/lib_ux/glyphs/icon_right.gif", bolos_sdk))
        .arg(&format!("{}/lib_ux/glyphs/icon_up.gif", bolos_sdk))
        .arg("--glyphcheader")
        .output()
        .expect("failed");

    let dest_path = Path::new(&main_path);
    let mut f = File::create(&dest_path.join("glyphs.h")).unwrap();
    f.write_all(&output.stdout).unwrap();

    println!("{:?}", output.stderr);
    assert!(output.status.success());

    cc::Build::new()
        .compiler("clang")
        .target("thumbv6m-none-eabi")
        .file("./src/c/src.c")
        .file("./src/c/sjlj.s")
        .file(format!("{}/src/os.c", bolos_sdk))
        // .file(format!("{}/src/os_printf.c", bolos_sdk))
        .file(format!("{}/src/os_io_seproxyhal.c", bolos_sdk))
        .file(format!("{}/src/os_io_usb.c", bolos_sdk))
        .file(format!("{}/src/pic_internal.c", bolos_sdk))
        .file(format!("{}/src/pic.c", bolos_sdk))
        .file(format!("{}/src/syscalls.c", bolos_sdk))
        .file(format!("{}/lib_ux/glyphs/glyphs.c", bolos_sdk))
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
        // The following flags should be the same as in wrapper
        //TODO : try to get rid of the flags in wrapper.h by using
        //      bindgen from within build.rs
        .define("ST31", None)
        .define("IO_HID_EP_LENGTH", Some("64"))
        .define("USB_SEGMENT_SIZE", Some("64"))
        .define("TARGET_NAME", Some("TARGET_NANOS"))
        .define("UI_NANO_S", None)
        .define("OS_IO_SEPROXYHAL", None)
        .define("HAVE_BAGL", None)
        .define("HAVE_IO_USB", None)
        .define("HAVE_L4_USBLIB", None)
        .define("HAVE_USB_APDU", None)
        .define("HAVE_UX_FLOW", None)
        .define("IO_USB_MAX_ENDPOINTS", Some("6"))
        .define("IO_SEPROXYHAL_BUFFER_SIZE_B", Some("128"))
        .include(gcc_toolchain)
        .include(format!("{}/include", bolos_sdk))
        .include(format!("{}/lib_ux/glyphs", bolos_sdk))
        .include(format!("{}/lib_ux/include", bolos_sdk))
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
        // More or less same flags as in the
        // C SDK Makefile.defines
        .no_default_flags(true)
        .pic(true)
        .flag("-fropi")
        .flag("--target=thumbv6m-none-eabi")
        .flag("-fomit-frame-pointer")
        // .flag("-momit-leaf-frame-pointer")

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
        .compile("rust-app");
}
