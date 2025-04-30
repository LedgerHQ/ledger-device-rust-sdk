use std::fmt::{Display, Formatter};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::str::from_utf8;

use cargo_metadata::{Message, Package};
use clap::{Parser, Subcommand, ValueEnum};
use serde_derive::Deserialize;
use serde_json::json;

use setup::install_targets;
use utils::*;

mod setup;
mod utils;

#[derive(Debug, Deserialize)]
struct LedgerMetadata {
    curve: Vec<String>,
    path: Vec<String>,
    flags: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeviceMetadata {
    icon: String,
    flags: Option<String>,
}

#[derive(Parser, Debug)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
#[clap(name = "Ledger devices build and load commands")]
#[clap(version = "0.0")]
#[clap(about = "Builds the project and emits a JSON manifest for ledgerctl.")]
enum Cli {
    Ledger(CliArgs),
}

#[derive(clap::Args, Debug)]
struct CliArgs {
    #[clap(long)]
    #[clap(value_name = "prebuilt ELF exe")]
    use_prebuilt: Option<PathBuf>,

    #[clap(subcommand)]
    command: MainCommand,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq)]
enum Device {
    Nanox,
    Nanosplus,
    Stax,
    Flex,
}

impl Display for Device {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl AsRef<str> for Device {
    fn as_ref(&self) -> &str {
        match self {
            Device::Nanox => "nanox",
            Device::Nanosplus => "nanosplus",
            Device::Stax => "stax",
            Device::Flex => "flex",
        }
    }
}

#[derive(Subcommand, Debug)]
enum MainCommand {
    #[clap(about = "install custom target files")]
    Setup,
    #[clap(about = "build the project for a given device")]
    Build {
        #[clap(value_enum)]
        #[clap(help = "device to build for")]
        device: Device,
        #[clap(short, long)]
        #[clap(help = "load on a device")]
        load: bool,
        #[clap(last = true)]
        remaining_args: Vec<String>,
    },
}

fn main() {
    let Cli::Ledger(cli) = Cli::parse();

    match cli.command {
        MainCommand::Setup => install_targets(),
        MainCommand::Build {
            device: d,
            load: a,
            remaining_args: r,
        } => {
            build_app(d, a, cli.use_prebuilt, r);
        }
    }
}

fn retrieve_metadata(
    device: Device,
    manifest_path: Option<&str>,
) -> Result<(Package, LedgerMetadata, DeviceMetadata), ()> {
    let mut cmd = cargo_metadata::MetadataCommand::new();

    // Only used during tests
    if let Some(manifestpath) = manifest_path {
        cmd = cmd.manifest_path(manifestpath).clone();
    }

    let res = cmd
        .no_deps()
        .exec()
        .expect("Could not execute `cargo metadata`");

    let this_pkg = res.packages.last().unwrap();
    let metadata_section = this_pkg.metadata.get("ledger");

    if let Some(metadatasection) = metadata_section {
        let metadata_device = metadata_section
            .unwrap()
            .clone()
            .get(device.as_ref())
            .unwrap()
            .clone();

        let ledger_metadata: LedgerMetadata =
            serde_json::from_value(metadatasection.clone())
                .expect("Could not deserialize medatada.ledger");
        let device_metadata: DeviceMetadata =
            serde_json::from_value(metadata_device)
                .expect("Could not deserialize device medatada");

        Ok((this_pkg.clone(), ledger_metadata, device_metadata))
    } else {
        println!("No metadata found for device: {}", device);
        Err(())
    }
}

fn build_app(
    device: Device,
    is_load: bool,
    use_prebuilt: Option<PathBuf>,
    remaining_args: Vec<String>,
) {
    let exe_path = match use_prebuilt {
        None => {
            let c_sdk_path = match device {
                Device::Nanosplus => std::env::var("NANOSP_SDK"),
                Device::Nanox => std::env::var("NANOX_SDK"),
                Device::Stax => std::env::var("STAX_SDK"),
                Device::Flex => std::env::var("FLEX_SDK"),
            };

            let mut args: Vec<String> = vec![];
            match std::env::var("RUST_NIGHTLY") {
                Ok(version) => {
                    println!("Use Rust nightly toolchain: {}", version);
                    args.push(format!("+{}", version))
                }
                Err(_) => {
                    let rustup_cmd =
                        Command::new("rustup").arg("default").output().unwrap();
                    println!(
                        "Use Rust default toolchain: {}",
                        from_utf8(rustup_cmd.stdout.as_slice()).unwrap()
                    );
                }
            }
            args.push(String::from("build"));
            args.push(String::from("--release"));
            args.push(format!("--target={}", device.as_ref()));
            args.push(String::from(
                "--message-format=json-diagnostic-rendered-ansi",
            ));

            match std::env::var("LEDGER_SDK_PATH") {
                Ok(_) => (),
                Err(_) => match c_sdk_path {
                    Ok(path) => args.push(format!(
                        "--config=env.LEDGER_SDK_PATH=\"{}\"",
                        path
                    )),
                    Err(_) => println!("C SDK will have to be cloned"),
                },
            }

            let mut cargo_cmd = Command::new("cargo")
                .args(args)
                .args(&remaining_args)
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();

            let mut exe_path = PathBuf::new();
            let out = cargo_cmd.stdout.take().unwrap();
            let reader = std::io::BufReader::new(out);
            for message in Message::parse_stream(reader) {
                match message.as_ref().unwrap() {
                    Message::CompilerArtifact(artifact) => {
                        if let Some(n) = &artifact.executable {
                            exe_path = n.to_path_buf();
                        }
                    }
                    Message::CompilerMessage(message) => {
                        println!("{message}");
                    }
                    _ => (),
                }
            }

            cargo_cmd.wait().expect("Couldn't get cargo's exit status");

            exe_path
        }
        Some(prebuilt) => prebuilt.canonicalize().unwrap(),
    };

    let (this_pkg, metadata_ledger, metadata_device) =
        retrieve_metadata(device, None).unwrap();

    let package_path = this_pkg
        .manifest_path
        .parent()
        .expect("Could not find package's parent path");

    /* exe_path = "exe_parent" + "exe_name" */
    let exe_name = exe_path.file_name().unwrap();
    let exe_parent = exe_path.parent().unwrap();

    let hex_file_abs = exe_path
        .parent()
        .unwrap()
        .join(exe_name)
        .with_extension("hex");

    let hex_file = hex_file_abs.strip_prefix(exe_parent).unwrap();

    export_binary(&exe_path, &hex_file_abs);

    // app.json will be placed next to hex file
    let app_json_name = format!("app_{}.json", device.as_ref());
    let app_json = exe_parent.join(app_json_name);

    // Retrieve real data size and SDK infos from ELF
    let infos = retrieve_infos(&exe_path).unwrap();

    let flags = match metadata_device.flags {
        Some(flags) => flags,
        None => match metadata_ledger.flags {
            Some(flags) => match device {
                // Modify flags to enable BLE if targeting Nano X
                Device::Nanosplus => flags,
                Device::Nanox | Device::Stax | Device::Flex => {
                    let base =
                        u32::from_str_radix(flags.trim_start_matches("0x"), 16)
                            .unwrap_or(0);
                    format!("0x{:x}", base | 0x200)
                }
            },
            None => String::from("0x000"),
        },
    };

    // Target ID according to target, in case it
    // is not present in the retrieved ELF infos.
    let backup_targetid: String = match device {
        Device::Nanox => String::from("0x33000004"),
        Device::Nanosplus => String::from("0x33100004"),
        Device::Stax => String::from("0x33200004"),
        Device::Flex => String::from("0x33300004"),
    };

    // create manifest
    let file = fs::File::create(&app_json).unwrap();
    let mut json = json!({
        "name": metadata_ledger.name.as_ref().unwrap_or(&this_pkg.name),
        "version": &this_pkg.version,
        "icon": metadata_device.icon,
        "targetId": infos.target_id.unwrap_or(backup_targetid),
        "flags": flags,
        "derivationPath": {
            "curves": metadata_ledger.curve,
            "paths": metadata_ledger.path
        },
        "binary": hex_file,
        "dataSize": infos.size
    });

    json["apiLevel"] = infos.api_level.into();
    serde_json::to_writer_pretty(file, &json).unwrap();

    // Copy icon to the same directory as the app.json
    let icon_path = package_path.join(&metadata_device.icon);
    let icon_dest =
        exe_parent.join(&metadata_device.icon.split('/').last().unwrap());

    fs::copy(icon_path, icon_dest).unwrap();

    // Use ledgerctl to dump the APDU installation file.
    // Either dump to the location provided by the --out-dir cargo
    // argument if provided or use the default binary path.
    let output_dir: Option<PathBuf> = remaining_args
        .iter()
        .position(|arg| arg == "--out-dir" || arg.starts_with("--out-dir="))
        .and_then(|index| {
            let out_dir_arg = &remaining_args[index];
            // Extracting the value from "--out-dir=<some value>" or "--out-dir <some value>"
            if out_dir_arg.contains('=') {
                Some(out_dir_arg.split('=').nth(1).unwrap().to_string())
            } else {
                remaining_args
                    .get(index + 1)
                    .map(|path_str| path_str.to_string())
            }
        })
        .map(PathBuf::from);
    let exe_filename = exe_path.file_name().unwrap().to_str();
    let exe_parent = exe_path.parent().unwrap().to_path_buf();
    let apdu_file_path = output_dir
        .unwrap_or(exe_parent)
        .join(exe_filename.unwrap())
        .with_extension("apdu");
    dump_with_ledgerctl(
        package_path,
        &app_json,
        apdu_file_path.to_str().unwrap(),
    );

    if is_load {
        install_with_ledgerctl(package_path, &app_json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_metadata() {
        match retrieve_metadata(Device::Flex, Some("./tests/valid/Cargo.toml")) {
            Ok(res) => {
                let (_, metadata_ledger, _metadata_nanos) = res;
                assert_eq!(metadata_ledger.name, Some("TestApp".to_string()));
                assert_eq!(metadata_ledger.curve, ["secp256k1"]);
                assert_eq!(metadata_ledger.flags, Some(String::from("0x38")));
                assert_eq!(metadata_ledger.path, ["'44/123"]);
            },
            Err(_) => panic!("Failed to retrieve metadata"),
        };
    }

    #[test]
    fn valid_metadata_variant() {
        match retrieve_metadata(
            Device::Flex,
            Some("./tests/valid_variant/Cargo.toml"),
        ) {
            Ok(res) => {
                let (_, metadata_ledger, _metadata_nanos) = res;
                assert_eq!(metadata_ledger.name, Some("TestApp".to_string()));
                assert_eq!(metadata_ledger.curve, ["secp256k1"]);
                assert_eq!(metadata_ledger.flags, Some(String::from("0x38")));
                assert_eq!(metadata_ledger.path, ["'44/123"]);
            },
            Err(_) => panic!("Failed to retrieve metadata"),
        };
    }

    #[test]
    fn valid_outdated_metadata() {

        match retrieve_metadata(
            Device::Flex,
            Some("./tests/valid_outdated/Cargo.toml"),
        ) {
            Ok(res) => {
                let (_, metadata_ledger, _metadata_nanos) = res;
                assert_eq!(metadata_ledger.name, Some("TestApp".to_string()));
                assert_eq!(metadata_ledger.curve, ["secp256k1"]);
                assert_eq!(metadata_ledger.flags, Some(String::from("0x38")));
                assert_eq!(metadata_ledger.path, ["'44/123"]);
            },
            Err(_) => panic!("Failed to retrieve metadata"),
        };
    }
}
