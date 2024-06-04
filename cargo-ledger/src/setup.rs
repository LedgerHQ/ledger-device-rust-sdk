use std::path::Path;
use std::process::Command;

pub fn install_targets() {
    println!("[ ] Checking for installed custom targets...");
    // Check if target files are installed
    let sysroot_cmd = Command::new("rustc")
        .arg("--print")
        .arg("sysroot")
        .output()
        .expect("failed to call rustc")
        .stdout;
    let sysroot_cmd = std::str::from_utf8(&sysroot_cmd).unwrap().trim();

    let target_files_url = Path::new(
        "https://raw.githubusercontent.com/LedgerHQ/ledger-device-rust-sdk/a7fb841160df34b8de268b136704c8b2ed8f9973/ledger_device_sdk/"
    );
    let sysroot = Path::new(sysroot_cmd).join("lib").join("rustlib");

    // Retrieve each target file independently
    // TODO: handle target.json modified upstream
    for target in &["nanos", "nanox", "nanosplus", "stax", "flex"] {
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
