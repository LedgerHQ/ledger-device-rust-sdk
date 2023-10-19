use std::path::PathBuf;
use std::{env, error::Error};

fn extract_defines_from_cx_makefile(command: &mut cc::Build, makefile: &String) {
    let mut makefile = File::open(makefile).unwrap();
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
}

fn main() -> Result<(), Box<dyn Error>> {
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
