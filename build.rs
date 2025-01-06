#![allow(clippy::all)]

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

const LOOKUP_TABLE: &str = "LookupTable";

fn to_camel_case(input: &str) -> String {
    let mut result = String::new();
    let mut uppercase_next = true;

    for c in input.chars() {
        match c {
            '_' | ' ' => uppercase_next = true,
            '-' => uppercase_next = true, // Handles possible dashes as well
            c if uppercase_next => {
                result.push(c.to_ascii_uppercase());
                uppercase_next = false;
            }
            c => result.push(c.to_ascii_lowercase()),
        }
    }

    result
}

fn resolve_data_type(variant: u32) -> &'static str {
    match variant {
        0 => "crate::prelude::data_types::Uint16",
        1 => "crate::prelude::data_types::Uint32",
        2 => "crate::prelude::data_types::Int32",
        3 => "crate::prelude::data_types::Float32",
        4 => "crate::prelude::data_types::Uint64",
        99 => "crate::prelude::data_types::Byte",
        98 => "crate::prelude::data_types::Byte", // TODO: Support `String`.
        variant => panic!(
            "Unsupported data type: {}. Expected one-of 0,1,2,34,98,99.",
            variant
        ),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Path to the C header file
    let header_path = "./resources/LabJackMModbusMap.h";
    let out_dir = &env::var("OUT_DIR")?;
    let path = Path::new(out_dir).join("codegen.rs");
    let mut file = BufWriter::new(File::create(&path)?);

    // Open the C header file for reading
    let header_file = File::open(header_path)?;
    let reader = BufReader::new(header_file);

    let lines: Vec<(String, (Option<u32>, Option<u32>))> = reader
        .lines()
        .filter_map(|e| e.ok())
        .filter_map(|line| {
            if !line.contains("_ADDRESS") && !line.contains("_TYPE") {
                return None;
            }

            line.trim()
                .strip_prefix("enum { LJM_")
                .and_then(|v| v.strip_suffix("};"))
                .and_then(|v| v.split_once('='))
                .and_then(|(a, b)| {
                    if a.ends_with("_ADDRESS ") {
                        a.rsplit_once("_ADDRESS").map(|(key, _)| {
                            let addr = b.trim().parse::<u32>().ok();
                            if addr.is_none() {
                                eprintln!("Could not parse address (integer) of value: {}", b);
                            }
                            (to_camel_case(key), (addr, None))
                        })
                    } else {
                        a.rsplit_once("_TYPE").map(|(key, _)| {
                            let d_type = b.trim().parse::<u32>().ok();
                            if d_type.is_none() {
                                eprintln!("Could not parse datatype (integer) of value: {}", b);
                            }
                            (to_camel_case(key), (None, d_type))
                        })
                    }
                })
        })
        .collect::<Vec<_>>();

    let mut map: HashMap<String, (Option<u32>, Option<u32>)> = HashMap::new();

    // Group the entries by their string key and merge the options.
    for (key, (opt1, opt2)) in lines {
        // println!("cargo:warning={}, {:?}, {:?}", key, opt1, opt2);
        map.entry(key)
            .and_modify(|(existing1, existing2)| {
                *existing1 = existing1.or(opt1); // Merge first option.
                *existing2 = existing2.or(opt2); // Merge second option.
            })
            .or_insert((opt1, opt2));
    }

    writeln!(&mut file, "use serde::{{Deserialize, Serialize}};").unwrap();
    writeln!(
        &mut file,
        "#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, Serialize, Deserialize)]"
    )
    .unwrap();
    writeln!(&mut file, "pub enum {} {{", LOOKUP_TABLE).unwrap();
    for (key, _) in map.clone() {
        writeln!(&mut file, "    {},", key).unwrap();
    }
    writeln!(&mut file, "}}",).unwrap();

    // writeln!(&mut file, "impl From<{}> for (u32, u32) {{", LOOKUP_TABLE).unwrap();
    writeln!(&mut file, "impl {} {{", LOOKUP_TABLE).unwrap();
    // writeln!(
    //     &mut file,
    //     "    pub const fn raw(&self) -> crate::core::LabJackEntity<impl crate::prelude::data_types::Decode> {{",
    // )
    // .unwrap();
    // writeln!(&mut file, "        match self {{",).unwrap();
    // for (key, (address, data_type)) in &map {
    //     writeln!(
    //         &mut file,
    //         "            {}::{} => crate::core::LabJackEntity::<{}>::new({}, {}::{}),",
    //         LOOKUP_TABLE,
    //         key,
    //         resolve_data_type(data_type.ok_or(format!(
    //             "Could not decode given labjack data type for {}",
    //             key
    //         ))?),
    //         address.ok_or(format!(
    //             "Could not decode given labjack address for {}",
    //             key
    //         ))?,
    //         LOOKUP_TABLE,
    //         key,
    //     )
    //     .unwrap();
    // }
    // writeln!(&mut file, "         }}",).unwrap();
    // writeln!(&mut file, "    }}",).unwrap();

    writeln!(&mut file, "}}",).unwrap();

    for (key, (address, data_type)) in map {
        let dt = resolve_data_type(data_type.unwrap());
        let addr = address.unwrap();

        let content = format!(
            "crate::core::LabJackEntity::<{}>::new({}, {}::{})",
            resolve_data_type(data_type.ok_or(format!(
                "Could not decode given labjack data type for {}",
                key
            ))?),
            address.ok_or(format!(
                "Could not decode given labjack address for {}",
                key
            ))?,
            LOOKUP_TABLE,
            key,
        );

        writeln!(
            &mut file,
            r#"
            /// The {key} register.
            ///
            /// Data Type: **{dt}**
            ///
            /// Address: **{addr}**
            ///
            pub struct {key};

            impl crate::prelude::data_types::Register for {key} {{
                type DataType = {dt};
                const NAME: &'static str = "{key}";
                const ADDRESS: u16 = {addr};

                fn entity() -> crate::core::LabJackEntity<Self::DataType> {{
                    {content}
                }}
            }}
        "#
        )
        .unwrap();
    }

    Ok(())
}
