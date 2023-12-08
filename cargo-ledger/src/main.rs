use std::fmt::{Display, Formatter};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use cargo_metadata::{Message, Package};
use clap::{Parser, Subcommand, ValueEnum};
use serde_derive::Deserialize;
use serde_json::json;

use setup::install_targets;
use utils::*;

mod setup;
mod utils;

/// Structure for retrocompatibility, when the cargo manifest file
/// contains a single `[package.metadata.nanos]` section
#[derive(Debug, Deserialize)]
struct NanosMetadata {
    curve: Vec<String>,
    path: Vec<String>,
    flags: String,
    icon: String,
    icon_small: String,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LedgerMetadata {
    curve: Vec<String>,
    path: Vec<String>,
    flags: String,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeviceMetadata {
    icon: String,
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

    #[clap(long)]
    #[clap(help = concat!(
        "Should the app.hex be placed next to the app.json, or next to the input exe?",
        " ",
        "Typically used with --use-prebuilt when the input exe is in a read-only location.",
    ))]
    hex_next_to_json: bool,

    #[clap(subcommand)]
    command: MainCommand,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq)]
enum Device {
    Nanos,
    Nanox,
    Nanosplus,
}

impl Display for Device {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl AsRef<str> for Device {
    fn as_ref(&self) -> &str {
        match self {
            Device::Nanos => "nanos",
            Device::Nanox => "nanox",
            Device::Nanosplus => "nanosplus",
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
            build_app(d, a, cli.use_prebuilt, cli.hex_next_to_json, r);
        }
    }
}

fn retrieve_metadata(
    device: Device,
    manifest_path: Option<&str>,
) -> (Package, LedgerMetadata, DeviceMetadata) {
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

        (this_pkg.clone(), ledger_metadata, device_metadata)
    } else {
        println!("WARNING: 'package.metadata.ledger' section is missing in Cargo.toml, trying 'package.metadata.nanos'");
        let nanos_section = this_pkg.metadata.get("nanos").expect(
            "No appropriate [package.metadata.<ledger|nanos>] section found.",
        );

        let nanos_metadata: NanosMetadata =
            serde_json::from_value(nanos_section.clone())
                .expect("Could not deserialize medatada.nanos");
        let ledger_metadata = LedgerMetadata {
            curve: nanos_metadata.curve,
            path: nanos_metadata.path,
            flags: nanos_metadata.flags,
            name: nanos_metadata.name,
        };

        let device_metadata = DeviceMetadata {
            icon: match device {
                Device::Nanos => nanos_metadata.icon,
                _ => nanos_metadata.icon_small,
            },
        };

        (this_pkg.clone(), ledger_metadata, device_metadata)
    }
}

fn build_app(
    device: Device,
    is_load: bool,
    use_prebuilt: Option<PathBuf>,
    hex_next_to_json: bool,
    remaining_args: Vec<String>,
) {
    let exe_path = match use_prebuilt {
        None => {
            let mut cargo_cmd = Command::new("cargo")
                .args([
                    "build",
                    "--release",
                    format!("--target={}", device.as_ref()).as_str(),
                    "--message-format=json-diagnostic-rendered-ansi",
                ])
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
        retrieve_metadata(device, None);
    let current_dir = this_pkg
        .manifest_path
        .parent()
        .expect("Could not find package's parent path");

    let hex_file_abs = if hex_next_to_json {
        current_dir
    } else {
        exe_path.parent().unwrap()
    }
    .join("app.hex");

    export_binary(&exe_path, &hex_file_abs);

    // app.json will be placed in the app's root directory
    let app_json_name = format!("app_{}.json", device.as_ref());
    let app_json = current_dir.join(app_json_name);

    // Find hex file path relative to 'app.json'
    let hex_file = hex_file_abs.strip_prefix(current_dir).unwrap();

    // Retrieve real data size and SDK infos from ELF
    let infos = retrieve_infos(&exe_path).unwrap();

    // Modify flags to enable BLE if targeting Nano X
    let flags = match device {
        Device::Nanos | Device::Nanosplus => metadata_ledger.flags,
        Device::Nanox => {
            let base = u32::from_str_radix(metadata_ledger.flags.as_str(), 16)
                .unwrap_or(0);
            format!("0x{:x}", base | 0x200)
        }
    };

    // Target ID according to target, in case it
    // is not present in the retrieved ELF infos.
    let backup_targetid : String = match device {
        Device::Nanos => String::from("0x31100004"),
        Device::Nanox => String::from("0x33000004"),
        Device::Nanosplus => String::from("0x33100004"),
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
    // Ignore apiLevel for Nano S as it is unsupported for now
    match device {
        Device::Nanos => (),
        _ => {
            json["apiLevel"] = infos.api_level.into();
        }
    }
    serde_json::to_writer_pretty(file, &json).unwrap();

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
        .map(|path_str| PathBuf::from(path_str));
    let exe_filename = exe_path.file_name().unwrap().to_str();
    let exe_parent = exe_path.parent().unwrap().to_path_buf();
    let apdu_file_path = output_dir.unwrap_or(exe_parent).join(exe_filename.unwrap()).with_extension("apdu");
    dump_with_ledgerctl(current_dir, &app_json, apdu_file_path.to_str().unwrap());

    if is_load {
        install_with_ledgerctl(current_dir, &app_json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_metadata() {
        let (_, metadata_ledger, metadata_nanos) =
            retrieve_metadata(Device::Nanos, Some("./tests/valid/Cargo.toml"));

        assert_eq!(metadata_ledger.name, Some("TestApp".to_string()));
        assert_eq!(metadata_ledger.curve, ["secp256k1"]);
        assert_eq!(metadata_ledger.flags, "0x38");
        assert_eq!(metadata_ledger.path, ["'44/123"]);

        assert_eq!(metadata_nanos.icon, "./assets/nanos.gif")
    }

    #[test]
    fn valid_metadata_variant() {
        let (_, metadata_ledger, metadata_nanos) = retrieve_metadata(
            Device::Nanos,
            Some("./tests/valid_variant/Cargo.toml"),
        );

        assert_eq!(metadata_ledger.name, Some("TestApp".to_string()));
        assert_eq!(metadata_ledger.curve, ["secp256k1"]);
        assert_eq!(metadata_ledger.flags, "0x38");
        assert_eq!(metadata_ledger.path, ["'44/123"]);
        assert_eq!(metadata_nanos.icon, "./assets/nanos.gif")
    }

    #[test]
    fn valid_outdated_metadata() {
        let (_, metadata_ledger, metadata_nanos) = retrieve_metadata(
            Device::Nanos,
            Some("./tests/valid_outdated/Cargo.toml"),
        );

        assert_eq!(metadata_ledger.name, Some("TestApp".to_string()));
        assert_eq!(metadata_ledger.curve, ["secp256k1"]);
        assert_eq!(metadata_ledger.flags, "0");
        assert_eq!(metadata_ledger.path, ["'44/123"]);
        assert_eq!(metadata_nanos.icon, "nanos.gif")
    }
}
