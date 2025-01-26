#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use std::collections::HashMap;
use std::env;
use std::fmt::Write;
use std::fs;
use std::path::Path;

const CODEGEN_HEADER: &str = r#"// Codegen - @bennjii 2025 Sourced @ labjack/ljm_constants.json
// All required attributes
use crate::prelude::*;
// All access control variants to avoid duplication
use crate::prelude::AccessControl::*;

"#;

fn uppercase_to_pascal_case(input: &str) -> String {
    let mut words = input.split('_').filter(|w| !w.is_empty());
    let mut pascal_case = String::new();

    for word in words {
        if let Some(first_char) = word.chars().next() {
            pascal_case.push(first_char.to_ascii_uppercase());
            pascal_case.push_str(&word[1..].to_lowercase());
        }
    }

    pascal_case
}

fn main() {
    // Path to the ljm_constants.json file
    let input_file = "./resources/ljm_constants.json";
    let out_dir = env::var("OUT_DIR").unwrap();
    let output_file = Path::new(&out_dir).join("codegen.rs");

    // Read and parse the JSON file
    let content = fs::read_to_string(input_file).expect("Failed to read JSON file");
    let data: serde_json::Value =
        serde_json::from_str(&content).expect("Failed to parse JSON file");

    // Extract registers and generate output
    let mut output = String::new();

    // Write header
    output.write_str(CODEGEN_HEADER).unwrap();

    let support_map = crate::SupportLookup::try_from(&data)
        .expect("Could not decode support information, improper format.");

    output
        .write_str(&format!(
            r#"
// LabJack Constants Version: {}
// Support URL: {}
"#,
            support_map.version, support_map.support_url
        ))
        .unwrap();

    let mut all_register_names = vec![];

    if let Some(registers) = data.get("registers").and_then(|r| r.as_array()) {
        for reg in registers {
            // We cannot proceed if these properties do not exist, hence panic.
            let name = reg.get("name").unwrap().as_str().unwrap();
            let address = reg.get("address").unwrap().as_u64().unwrap();
            let r#type = reg.get("type").unwrap().as_str().unwrap();
            let desc = reg
                .get("description")
                .map(|v| v.as_str().unwrap())
                .unwrap_or("No Description");

            let tags = reg
                .get("tags")
                .map(|tags| tags.as_array().unwrap().to_vec())
                .unwrap_or(vec![])
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect::<Vec<_>>();

            let access_control =
                crate::AccessControl::try_from(reg.get("readwrite").unwrap().as_str().unwrap())
                    .unwrap();

            let all_devices = reg.get("devices").unwrap().as_array().unwrap();
            let mut devices = vec![];
            for device in all_devices {
                devices.push(
                    crate::DeviceCompat::try_from(device).unwrap_or_else(|_| {
                        panic!("Could not deserialise device note: {device:?}")
                    }),
                );
            }

            if let Some((base_name, range, suffix)) =
                parse_name_with_range_and_optional_suffix(name)
            {
                for i in range {
                    let expanded_name = if suffix.is_empty() {
                        format!("{}{}", base_name, i)
                    } else {
                        format!("{}{}_{}", base_name, i, suffix)
                    };

                    all_register_names.push(expanded_name.clone());

                    generate_register(
                        &mut output,
                        Register {
                            data_type: decode_type(r#type),
                            name: expanded_name.as_str(),
                            base_address: address,
                            offset: Some(i),
                            access_control: &access_control,
                            desc,
                            devices: &devices,
                            tags: &tags,
                            default: None,
                        },
                        &support_map,
                    );
                }
            } else {
                all_register_names.push(name.to_string());

                // The case when the register does not contain
                generate_register(
                    &mut output,
                    Register {
                        name,
                        base_address: address,
                        data_type: decode_type(r#type),
                        offset: None,
                        access_control: &access_control,
                        desc,
                        devices: &devices,
                        tags: &tags,
                        default: None,
                    },
                    &support_map,
                );
            }
        }
    }

    output.push_str(
        r#"
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum RegisterList {
"#,
    );
    for reg in all_register_names {
        output.push_str(&format!(
            "\t{},\n",
            uppercase_to_pascal_case(&reg.to_uppercase())
        ));
    }
    output.push_str("}");

    // Write the generated code to the output file
    fs::write(&output_file, output).expect("Failed to write output file");
    println!("cargo:rerun-if-changed={}", input_file);
}

fn decode_type(r#type: &str) -> &'static str {
    match r#type {
        "INT32" => "Int32",
        "UINT16" => "Uint16",
        "UINT32" => "Uint32",
        "UINT64" => "Uint64",
        "FLOAT32" => "Float32",
        "STRING" | "BYTE" => "Byte",
        _ => panic!("Unsupported type {}", r#type),
    }
}

fn size_of(data_type: &'static str) -> u64 {
    match data_type {
        "Byte" => 1,
        "Uint16" => 1,
        "Float32" => 2,
        "Uint32" => 2,
        "Int32" => 2,
        "Uint64" => 4,
        _ => panic!("Unsupported type {}", data_type),
    }
}

fn format_device_compat(compat: &DeviceCompat) -> String {
    format!(
        "  * - {}{} {}",
        compat.name,
        compat
            .min_firmware
            .map(|v| format!(" [Since {}]", v))
            .unwrap_or_default(),
        compat.desc.clone().unwrap_or("".to_string())
    )
}

fn generate_register(
    output: &mut String,
    Register {
        name,
        base_address,
        offset,
        data_type,
        access_control,
        desc,
        devices,
        tags,
        default,
    }: Register,
    support_lookup: &SupportLookup,
) {
    let control_value = match access_control {
        AccessControl::ReadOnly => "{ ReadableCtrl as u8 }",
        AccessControl::ReadWrite => "{ AllCtrl as u8 }",
        AccessControl::WriteOnly => "{ WritableCtrl as u8 }",
    };

    output.push_str(&format!(
        r#"
/**
  * #### {name}
  *
  * **Access**: {access_control:?} \
  * **Compatible Devices**:
{}
  *
  * {}
  *
  * _Relevant Documentation:_
  * {}
  */
pub const {}: AccessLimitedRegister<{control_value}> = AccessLimitedRegister {{
    register: Register {{
        name: RegisterList::{},
        address: {},
        data_type: LabJackDataType::{data_type},
        default_value: {default:?}
    }}
}};
"#,
        devices
            .iter()
            .map(format_device_compat)
            .collect::<Vec<_>>()
            .join("\n"),
        // "\" to delimit the end, \n to CR, and "  * " to indent and space.
        desc.replace(";", " \\\n  * "),
        tags.iter()
            .filter_map(|tag| {
                support_lookup.hashmap.get(tag).map(|support_suffix| {
                    format!("[{tag}]({}{})", support_lookup.base_url, support_suffix)
                })
            })
            .collect::<Vec<_>>()
            .join(", "),
        name.to_uppercase(),
        uppercase_to_pascal_case(&name),
        base_address + (offset.unwrap_or(0) * size_of(data_type)),
    ));
}

// Parses a register name with a range and optional suffix, e.g., "AIN#(0:149)_EF_READ_C"
fn parse_name_with_range_and_optional_suffix(
    name: &str,
) -> Option<(&str, std::ops::RangeInclusive<u64>, &str)> {
    let range_start = name.find("#(")?;
    let range_end = name.find(")")?;
    let suffix_start = range_end + 1;

    let base_name = &name[..range_start];
    let range_part = &name[(range_start + 2)..range_end];
    let suffix = name
        .get(suffix_start..)
        .unwrap_or("")
        .trim_start_matches('_'); // Handle optional suffix

    let mut parts = range_part.split(':');
    let start: u64 = parts.next()?.parse().ok()?;
    let end: u64 = parts.next()?.parse().ok()?;

    Some((base_name, start..=end, suffix))
}

#[derive(Debug)]
pub enum AccessControl {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

impl TryFrom<&str> for AccessControl {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "R" => Ok(AccessControl::ReadOnly),
            "W" => Ok(AccessControl::WriteOnly),
            "RW" => Ok(AccessControl::ReadWrite),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct DeviceCompat {
    pub min_firmware: Option<f64>,
    pub desc: Option<String>,
    pub name: String,
}

impl<'a> TryFrom<&'a serde_json::Value> for DeviceCompat {
    type Error = ();

    fn try_from(value: &'a serde_json::Value) -> Result<Self, Self::Error> {
        match value.is_object() {
            true => {
                // Expecting the form: { "device": string, "fwmin": float }
                let name = value.get("device").ok_or(())?.as_str().ok_or(())?;
                let min_firmware = value.get("fwmin").map(|val| val.as_f64().unwrap());
                let description = value
                    .get("description")
                    .map(|val| val.as_str().unwrap().to_string());

                Ok(DeviceCompat {
                    min_firmware,
                    name: name.to_string(),
                    desc: description,
                })
            }
            false => {
                let name = value.as_str().ok_or(())?;

                Ok(DeviceCompat {
                    min_firmware: None,
                    name: name.to_string(),
                    desc: None,
                })
            }
        }
    }
}

struct SupportLookup {
    version: String,
    support_url: String,
    base_url: String,
    hashmap: HashMap<String, String>,
}

impl TryFrom<&serde_json::Value> for SupportLookup {
    type Error = ();

    fn try_from(value: &serde_json::Value) -> Result<Self, Self::Error> {
        let header = value.get("header").ok_or(())?;
        let version = header.get("version").unwrap().as_str().unwrap();
        let support_url = header.get("support_url").unwrap().as_str().unwrap();
        let base_url = header.get("tags_base_url").unwrap().as_str().unwrap();

        let tags = value
            .get("tag_mappings")
            .unwrap()
            .as_object()
            .unwrap()
            .into_iter()
            .map(|(k, v)| (k.clone(), v.as_str().unwrap().to_string()))
            .collect::<HashMap<String, String>>();

        Ok(SupportLookup {
            version: version.to_string(),
            support_url: support_url.to_string(),
            base_url: base_url.to_string(),

            hashmap: tags,
        })
    }
}

#[derive(Debug)]
pub struct Register<'a> {
    pub name: &'a str,
    pub desc: &'a str,
    pub base_address: u64,
    pub offset: Option<u64>,
    pub data_type: &'static str,
    pub access_control: &'a AccessControl,
    pub devices: &'a Vec<DeviceCompat>,
    pub tags: &'a Vec<String>,
    pub default: Option<f64>,
}
