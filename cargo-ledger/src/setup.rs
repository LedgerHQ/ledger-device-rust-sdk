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
        "https://raw.githubusercontent.com/LedgerHQ/ledger-device-rust-sdk/cee5644d6c20ff97b13e79a30caca751b7b52ac8/ledger_device_sdk/",
    );
    let sysroot = Path::new(sysroot_cmd).join("lib").join("rustlib");

    // Retrieve each target file independently
    // TODO: handle target.json modified upstream
    for target in &["nanos", "nanox", "nanosplus"] {
        let outfilepath = sysroot.join(target).join("target.json");
        if !outfilepath.exists() {
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
        } else {
            println!("* {target} already installed");
        }
    }
}
