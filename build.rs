use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write, BufRead};
use std::path::Path;

fn main() {
    // Path to the C header file
    let header_path = "./resources/LabJackMModbusMap.h";
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    // Open the C header file for reading
    let header_file = File::open(header_path).expect("Cannot open header file");
    let reader = BufReader::new(header_file);

    let lines: Vec<(String, String)> = reader.lines()
        .filter_map(|e| e.ok())
        .filter_map(|line| {
            if !line.contains("_ADDRESS") {
                return None;
            }

            line.trim()
                .strip_prefix("enum { LJM_")
                .and_then(|v| v.strip_suffix("};"))
                .and_then(|v| v.split_once('='))
                .and_then(|(a, b)| {
                    a.split_once("_ADDRESS ")
                        .and_then(|(key, _)| {
                            Some((key.to_string(), b.to_string()))
                        })
                })
        })
        .collect::<Vec<_>>();

    let mut builder = phf_codegen::Map::new();
    for (key, value) in &lines {
        builder.entry(key, &value);
    }

    write!(
        &mut file,
        "pub static ADDRESSES: phf::Map<&'static str, u32> = {}",
        builder.build()
    ).unwrap();
    write!(&mut file, ";\n").unwrap();
}