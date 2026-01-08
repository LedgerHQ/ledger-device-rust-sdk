fn generate_install_parameters() {
    // Find the root package directory by looking at OUT_DIR
    // OUT_DIR is something like: /path/to/app/target/nanosplus/debug/build/ledger_device_sdk-xxx/out
    // We need to extract /path/to/app from this
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let out_path = std::path::PathBuf::from(&out_dir);

    // Navigate up from OUT_DIR to find the root: out -> build-hash -> build -> debug/release -> target-name -> target -> ROOT
    let root_dir = out_path
        .parent() // Remove /out
        .and_then(|p| p.parent()) // Remove /ledger_device_sdk-xxx
        .and_then(|p| p.parent()) // Remove /build
        .and_then(|p| p.parent()) // Remove /debug or /release
        .and_then(|p| p.parent()) // Remove /nanosplus (target name)
        .and_then(|p| p.parent()) // Remove /target
        .expect("Could not find root directory from OUT_DIR");

    println!("cargo:warning=Root directory: {}", root_dir.display());

    // Now run cargo metadata from the root directory
    let output = std::process::Command::new("cargo")
        .current_dir(root_dir)
        .args(&["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .expect("Failed to execute cargo metadata");

    let metadata_str = std::str::from_utf8(&output.stdout)
        .expect("Failed to convert cargo metadata output to UTF-8");
    let metadata = json::parse(metadata_str)
        .expect("Failed to parse cargo metadata output");

    println!("cargo:warning=Looking for ledger metadata...");

    // Check if packages exists and is an array
    if metadata["packages"].is_array() {
        for package in metadata["packages"].members() {
            let pkg_name = package["name"].as_str().unwrap_or("unknown");
            println!("cargo:warning=Checking package: {}", pkg_name);

            // Look for the ledger metadata section
            if !package["metadata"]["ledger"].is_null() {
                println!(
                    "cargo:warning=Found ledger metadata in package: {}",
                    pkg_name
                );
                
                // Fill APP_NAME environment variable (stored in ledger.app_name section in the ELF (see info.rs))
                let app_name = package["metadata"]["ledger"]["name"]
                    .as_str()
                    .expect("name not found");
                println!("cargo:rustc-env=APP_NAME={}", app_name);
                println!("cargo:warning=APP_NAME is {}", app_name);

                // Fill APP_FLAGS environment variable (stored in ledger.app_flags section in the ELF (see info.rs))
                let app_flags = package["metadata"]["ledger"]["flags"]
                    .as_str()
                    .expect("flags not found");
                println!("cargo:rustc-env=APP_FLAGS={}", app_flags);
                println!("cargo:warning=APP_FLAGS is {}", app_flags);

                // Generate install_params TLV blob (stored as install_parameters symbol in the ELF (see info.rs))
                let app_version = package["version"]
                    .as_str()
                    .expect("version not found");
                println!("cargo:rustc-env=APP_VERSION={}", app_version);
                println!("cargo:warning=APP_VERSION is {}", app_version);
                
                let curves = package["metadata"]["ledger"]["curve"]
                    .members()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>();
                println!("cargo:warning=curves are {:x?}", curves);
                
                let paths = package["metadata"]["ledger"]["path"]
                    .members()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>();
                println!("cargo:warning=paths are {:x?}", paths);

                // Handle optional path_slip21 field
                let paths_slip21: Vec<String> = if !package["metadata"]["ledger"]["path_slip21"].is_null() {
                    package["metadata"]["ledger"]["path_slip21"]
                        .members()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect()
                } else {
                    Vec::new()
                };

                if !paths_slip21.is_empty() {
                    println!("cargo:warning=paths_slip21 are {:x?}", paths_slip21);
                }

                let install_params_exe = match std::env::var("LEDGER_SDK_PATH") {
                    Ok(path) => format!("{}/install_params.py", path),
                    Err(_) => format!(
                        "/opt/{}-secure-sdk/install_params.py",
                        std::env::var_os("CARGO_CFG_TARGET_OS")
                            .unwrap()
                            .to_str()
                            .unwrap()
                    ),
                };
                let mut generate_tlv_install_params = std::process::Command::new("python3");
                generate_tlv_install_params.arg(install_params_exe.as_str());
                generate_tlv_install_params.arg("--appName").arg(app_name);
                generate_tlv_install_params
                    .arg("--appVersion")
                    .arg(app_version);
                curves.iter().for_each(|p| {
                    generate_tlv_install_params.arg("--curve").arg(p.as_str());
                });
                paths.iter().for_each(|p| {
                    generate_tlv_install_params.arg("--path").arg(p.as_str());
                });
                paths_slip21.iter().for_each(|p| {
                    generate_tlv_install_params
                        .arg("--path_slip21")
                        .arg(p.as_str());
                });
                let output = generate_tlv_install_params
                    .output()
                    .expect("Failed to execute install_params_generator");

                let tlv_blob = format!(
                    "[{}]",
                    std::str::from_utf8(output.stdout.as_slice())
                        .unwrap()
                        .trim()
                );

                // Parse the TLV blob and create temp txt files for inclusion (see app_info.rs)
                let bytes: Vec<u8> = tlv_blob
                    .trim_matches(|c| c == '[' || c == ']')
                    .split(',')
                    .filter_map(|s| {
                        let trimmed = s.trim();
                        if trimmed.is_empty() {
                            None
                        } else {
                            u8::from_str_radix(trimmed.trim_start_matches("0x"), 16).ok()
                        }
                    })
                    .collect();

                let byte_array_str = bytes
                    .iter()
                    .map(|b| format!("0x{:02x}", b))
                    .collect::<Vec<_>>()
                    .join(",");

                // Write to files in OUT_DIR for inclusion
                let out_dir = std::env::var("OUT_DIR").unwrap();

                // Write the array with brackets for direct inclusion
                std::fs::write(
                    std::path::Path::new(&out_dir).join("install_params.txt"),
                    format!("[{}]", byte_array_str),
                )
                .unwrap();

                std::fs::write(
                    std::path::Path::new(&out_dir).join("install_params_len.txt"),
                    bytes.len().to_string(),
                )
                .unwrap();

                println!("cargo:warning=INSTALL_PARAMS_BYTES is [{}]", byte_array_str);
                println!("cargo:warning=INSTALL_PARAMS_LEN is {}", bytes.len());

                // Exit early since we found the metadata
                return;
            }
        }
    }

    // If we get here, we didn't find any ledger metadata - this is OK for non-app builds
    println!("cargo:warning=No [package.metadata.ledger] section found - empty install parameters generation");
    // Write empty install parameters
    let out_dir = std::env::var("OUT_DIR").unwrap();
    std::fs::write(
        std::path::Path::new(&out_dir).join("install_params.txt"),
        "[]",
    )
    .unwrap();
    std::fs::write(
        std::path::Path::new(&out_dir).join("install_params_len.txt"),
        "0",
    )
    .unwrap();
}

fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");
    generate_install_parameters();
}
