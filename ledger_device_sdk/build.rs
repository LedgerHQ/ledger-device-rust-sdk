use std::env;
use std::path::PathBuf;
use std::process::Command;

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
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .expect("Failed to execute cargo metadata");

    let metadata: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("Failed to parse cargo metadata output");

    println!("cargo:warning=Looking for ledger metadata...");

    // Check if packages exists and is an array
    if let Some(packages) = metadata["packages"].as_array() {
        for package in packages {
            let pkg_name = package["name"].as_str().unwrap_or("unknown");
            println!("cargo:warning=Checking package: {}", pkg_name);

            // Look for the ledger metadata section
            if let Some(metadata_ledger) = package["metadata"]["ledger"].as_object() {
                println!(
                    "cargo:warning=Found ledger metadata in package: {}",
                    pkg_name
                );

                // Optional build variant (e.g. "testnet"). When selected, the table
                // [package.metadata.ledger.variants.<name>] is overlaid on top of the base
                // [package.metadata.ledger]: any key the variant defines wins, and everything
                // it omits (curve, flags, …) is inherited from the base. Selecting a variant
                // whose table is absent is a hard error — we never silently fall back to the
                // base (mainnet) values, since that could ship a "Testnet"-labelled binary
                // carrying mainnet derivation paths/curves (fail closed).
                let variant = resolve_variant();
                let overlay: Option<&serde_json::Map<String, serde_json::Value>> = match &variant {
                    Some(name) => {
                        println!("cargo:warning=Building variant `{}`", name);
                        let table = metadata_ledger
                            .get("variants")
                            .and_then(|v| v.get(name.as_str()))
                            .and_then(|v| v.as_object())
                            .unwrap_or_else(|| {
                                panic!(
                                    "package `{pkg_name}`: build variant `{name}` selected but \
                                     [package.metadata.ledger.variants.{name}] is missing or is not a table"
                                )
                            });
                        Some(table)
                    }
                    None => None,
                };

                // Resolve a top-level key: the variant overlay wins, else the base table.
                let lookup = |key: &str| -> Option<&serde_json::Value> {
                    overlay
                        .and_then(|o| o.get(key))
                        .or_else(|| metadata_ledger.get(key))
                };

                // Get device name
                let device = env::var_os("CARGO_CFG_TARGET_OS").unwrap();
                let device_name = device.to_str().unwrap();
                println!("cargo:warning=Device is {}", device_name);

                // Fill APP_NAME environment variable (stored in ledger.app_name section in the ELF (see app_info.rs))
                let app_name = lookup("name")
                    .and_then(|v| v.as_str())
                    .expect("name not found");
                println!("cargo:rustc-env=APP_NAME={}", app_name);
                println!("cargo:warning=APP_NAME is {}", app_name);

                // Fill APP_FLAGS environment variable (stored in ledger.app_flags section in the ELF (see app_info.rs))
                // APPLICATION_FLAG_BOLOS_SETTINGS, see ledger-secure-sdk/include/appflags.h.
                // Required on these devices but not on nanosplus (Bluetooth enabling).
                const APPLICATION_FLAG_BOLOS_SETTINGS: u32 = 0x200;
                let flags = lookup("flags")
                    .and_then(|v| v.as_str())
                    .expect("flags not found");
                let app_flags = match device_name {
                    "nanosplus" => String::from(flags),
                    "nanox" | "stax" | "flex" | "apex_p" => {
                        let base = u32::from_str_radix(flags.trim_start_matches("0x"), 16)
                            .unwrap_or_else(|_| {
                                panic!(
                                    "package `{pkg_name}`: ledger.flags must be a hex string like \"0x200\", got {flags:?}"
                                )
                            });
                        format!("0x{:x}", base | APPLICATION_FLAG_BOLOS_SETTINGS)
                    }
                    other => panic!("Unsupported device target_os: {other:?}"),
                };

                println!("cargo:rustc-env=APP_FLAGS={}", app_flags);
                println!("cargo:warning=APP_FLAGS is {}", app_flags);

                // Generate install_params TLV blob (stored as install_parameters symbol in the ELF (see app_info.rs))
                let app_version = package["version"].as_str().expect("version not found");
                println!("cargo:rustc-env=APP_VERSION={}", app_version);
                println!("cargo:warning=APP_VERSION is {}", app_version);

                let curves = lookup("curve")
                    .and_then(|v| v.as_array())
                    .expect("curves not found")
                    .iter()
                    .map(|v| v.as_str().unwrap().to_string())
                    .collect::<Vec<_>>();
                println!("cargo:warning=curves are {:x?}", curves);

                let paths = lookup("path")
                    .and_then(|v| v.as_array())
                    .expect("paths not found")
                    .iter()
                    .map(|v| v.as_str().unwrap().to_string())
                    .collect::<Vec<_>>();
                println!("cargo:warning=paths are {:x?}", paths);

                // Handle optional path_slip21 field
                let paths_slip21: Vec<String> = lookup("path_slip21")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect()
                    })
                    .unwrap_or_default();

                if !paths_slip21.is_empty() {
                    println!("cargo:warning=paths_slip21 are {:x?}", paths_slip21);
                }

                // Handle icon: a variant may override the per-device icon; if it does not,
                // the base [package.metadata.ledger.<device>] icon is used (icons are
                // cosmetic, so inheriting the base icon is acceptable).
                let icon = overlay
                    .and_then(|o| o.get(device_name))
                    .and_then(|device_metadata| device_metadata.get("icon"))
                    .and_then(|icon| icon.as_str())
                    .or_else(|| {
                        metadata_ledger
                            .get(device_name)
                            .and_then(|device_metadata| device_metadata.get("icon"))
                            .and_then(|icon| icon.as_str())
                    })
                    .unwrap_or_else(|| {
                        panic!(
                            "missing Ledger app icon metadata for device `{}`; expected \
                             `package.metadata.ledger.{device}.icon` to be a string, for \
                              example: [package.metadata.ledger.{device}] icon = \"path/to/icon.gif\"",
                            device_name,
                            device = device_name
                        )
                });
                println!("cargo:warning=APP_ICON is {}", icon);

                let c_sdk_path = resolve_c_sdk_path(device_name);
                println!("cargo:warning=C SDK path is {}", c_sdk_path.display());

                let icon_hex_string = convert_icon_to_hex(&c_sdk_path, device_name, root_dir, icon);

                // Now we have all the parameters, we can call the install_params.py script to generate the TLV blob
                let install_params_exe = c_sdk_path.join("install_params.py");
                let mut generate_tlv_install_params = std::process::Command::new("python3");
                generate_tlv_install_params.arg(&install_params_exe);
                generate_tlv_install_params.arg("--appName").arg(app_name);
                generate_tlv_install_params
                    .arg("--appVersion")
                    .arg(app_version);
                curves.iter().for_each(|p| {
                    generate_tlv_install_params.arg("--curve").arg(p.as_str());
                });
                paths.iter().for_each(|p| {
                    generate_tlv_install_params
                        .arg("--path")
                        .arg(p.as_str().trim_end_matches('/'));
                });
                paths_slip21.iter().for_each(|p| {
                    generate_tlv_install_params
                        .arg("--path_slip21")
                        .arg(p.as_str());
                });
                generate_tlv_install_params
                    .arg("--icon")
                    .arg(icon_hex_string);

                let output = generate_tlv_install_params
                    .output()
                    .expect("Failed to execute install_params_generator");

                if !output.status.success() {
                    panic!(
                        "call to install_params.py failed: {}",
                        std::str::from_utf8(&output.stderr).unwrap()
                    );
                }

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
    println!(
        "cargo:warning=No [package.metadata.ledger] section found - empty install parameters generation"
    );
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

/// Resolve the C SDK root path for the given device.
///
/// Uses `LEDGER_SDK_PATH` if set, otherwise falls back to the
/// device-specific environment variable (e.g. `NANOSPLUS_SDK`).
fn resolve_c_sdk_path(device_name: &str) -> PathBuf {
    PathBuf::from(env::var("LEDGER_SDK_PATH").unwrap_or_else(|_| {
        let var = match device_name {
            "nanosplus" => "NANOSP_SDK",
            "nanox" => "NANOX_SDK",
            "stax" => "STAX_SDK",
            "flex" => "FLEX_SDK",
            "apex_p" => "APEX_P_SDK",
            _ => panic!("Unsupported device: {}", device_name),
        };
        env::var(var).unwrap_or_else(|_| panic!("{} not set", var))
    }))
}

/// Run `icon2glyph.py` to convert the app icon into a hex string
/// suitable for the install-parameters TLV blob.
fn convert_icon_to_hex(
    c_sdk_path: &std::path::Path,
    device_name: &str,
    root_dir: &std::path::Path,
    icon: &str,
) -> String {
    let icon_hex_file = PathBuf::from(env::var("OUT_DIR").unwrap()).join("icon.hex");
    println!(
        "cargo:warning=Output file for icon2glyph is {}",
        icon_hex_file.display()
    );

    let icon2glyph = c_sdk_path.join("lib_nbgl/tools/icon2glyph.py");
    let mut cmd = Command::new("python3");
    cmd.arg(&icon2glyph);
    cmd.arg("--hexbitmap").arg(&icon_hex_file);
    if device_name == "nanosplus" || device_name == "nanox" {
        cmd.arg("--reverse");
    }
    cmd.arg(root_dir.join(icon));

    let output = cmd.output().expect("Failed to execute icon2glyph.py");
    if !output.status.success() {
        panic!(
            "call to icon2glyph.py failed: {}",
            std::str::from_utf8(&output.stderr).unwrap()
        );
    }
    std::fs::read_to_string(&icon_hex_file)
        .expect("Failed to read icon hex file")
        .trim()
        .to_string()
}

/// Maximum number of build variant slots exposed by the SDK (`variant_0` … `variant_9`).
const MAX_VARIANTS: u8 = 10;

/// Resolve the optional build variant to overlay onto `[package.metadata.ledger]`.
///
/// The SDK exposes generic, numbered variant features `variant_0` … `variant_9`. An app
/// forwards its own human-named feature to one of these slots, e.g.
/// `variant_testnet = ["ledger_device_sdk/variant_0"]`, and supplies the matching metadata
/// table `[package.metadata.ledger.variants.0]`. The slot number is the channel between the
/// app and this build script: a build script only receives `CARGO_FEATURE_<NAME>` for
/// features enabled on its *own* crate, so it cannot observe the app's feature name — only
/// which numbered SDK slot was switched on.
///
/// Enabling more than one `variant_N` feature is a hard error: we must never silently ship
/// one of several conflicting variants.
///
/// Returns `None` for a normal (base / mainnet) build.
fn resolve_variant() -> Option<String> {
    let mut selected: Option<u8> = None;
    for i in 0..MAX_VARIANTS {
        if env::var_os(format!("CARGO_FEATURE_VARIANT_{i}")).is_some() {
            if let Some(prev) = selected {
                panic!(
                    "multiple Ledger build variants selected (variant_{prev} and variant_{i}); \
                     enable at most one variant_N feature on `ledger_device_sdk`"
                );
            }
            selected = Some(i);
        }
    }
    selected.map(|i| i.to_string())
}

fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");
    generate_install_parameters();
}
