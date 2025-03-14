use std::path::Path;
use std::process::Command;
use std::str::from_utf8;

pub fn install_targets() {
    println!("[ ] Install custom targets...");
    // Check if target files are installed
    let mut args: Vec<String> = vec![];
    match std::env::var("RUST_NIGHTLY") {
        Ok(version) => {
            println!(
                "Install custom targets for nightly toolchain: {}",
                version
            );
            args.push(format!("+{}", version));
        }
        Err(_) => {
            let rustup_cmd =
                Command::new("rustup").arg("default").output().unwrap();
            println!(
                "Install custom targets for default toolchain {}",
                from_utf8(rustup_cmd.stdout.as_slice()).unwrap()
            );
        }
    }
    args.push(String::from("--print"));
    args.push(String::from("sysroot"));
    let sysroot_cmd = Command::new("rustc")
        .args(&args)
        .output()
        .expect("failed to call rustc")
        .stdout;
    let sysroot_cmd = std::str::from_utf8(&sysroot_cmd).unwrap().trim();

    let target_files_url = Path::new(
        "https://raw.githubusercontent.com/LedgerHQ/ledger-device-rust-sdk/refs/tags/ledger_secure_sdk_sys%401.7.0/ledger_secure_sdk_sys"
    );
    let sysroot = Path::new(sysroot_cmd).join("lib").join("rustlib");

    // Retrieve each target file independently
    // TODO: handle target.json modified upstream
    for target in &["nanox", "nanosplus", "stax", "flex"] {
        let outfilepath = sysroot.join(target).join("target.json");
        let targetpath =
            outfilepath.clone().into_os_string().into_string().unwrap();
        println!("* Adding \x1b[1;32m{target}\x1b[0m in \x1b[1;33m{targetpath}\x1b[0m");

        let target_url = target_files_url.join(format!("{target}.json"));
        let cmd = Command::new("curl")
            .arg(target_url)
            .arg("-o")
            .arg(outfilepath)
            .arg("--create-dirs")
            .output()
            .expect("failed to execute 'curl'");
        println!("{}", std::str::from_utf8(&cmd.stderr).unwrap());
    }

    // Install link_wrap.sh script needed for relocation
    println!("[ ] Install custom link script...");

    /*  Shall be put at the same place as rust-lld */
    let custom_link_script = "link_wrap.sh";

    let cmd = Command::new("find")
        .arg(sysroot_cmd)
        .arg("-name")
        .arg("rust-lld")
        .output()
        .expect("failed to find rust-lld linker")
        .stdout;

    let rust_lld_path = std::str::from_utf8(&cmd).unwrap();
    let end = rust_lld_path.rfind('/').unwrap();

    let outfilepath =
        sysroot.join(&rust_lld_path[..end]).join(custom_link_script);

    /* Retrieve the linker script */
    let target_url = target_files_url.join(custom_link_script);
    Command::new("curl")
        .arg(target_url)
        .arg("-o")
        .arg(&outfilepath)
        .output()
        .expect("failed to execute 'curl'");

    println!("* Custom link script is {}", outfilepath.display());

    /* Make the linker script executable */
    Command::new("chmod")
        .arg("+x")
        .arg(outfilepath)
        .output()
        .expect("failed to execute chmod");
}
