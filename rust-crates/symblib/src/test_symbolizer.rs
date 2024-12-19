#![allow(missing_docs)]

use std::io::Cursor;


// pub mod greeter {
//     include!(concat!(env!("OUT_DIR"), "/greeter.rs"));
// }
#[allow(clippy::all, non_snake_case)]
// Include compiled proto files, organizes the generated code into modules.
pub mod opentelemetry {
    pub mod proto {
        /// Service stub and clients
        // pub mod collector {
        //
        //     pub mod profiles {
        //         pub mod v1development {
        //             include!(concat!(env!("OUT_DIR"), "/opentelemetry.proto.profiles.v1development.rs"));
        //
        //             // tonic::include_proto!("opentelemetry.proto.collector.profiles.v1development");
        //             // tonic::include_proto!("opentelemetry.proto.collector.profiles.v1development.ProfilesService");
        //         }
        //     }
        // }
        // opentelemetrytlp/collector/profiles/v1development
        pub mod collector {
            pub mod profiles {
                pub mod v1development {
                    include!(concat!(env!("OUT_DIR"), "/opentelemetry.proto.collector.profiles.v1development.rs"));
                }
            }
        }
        /// Common types used across all signals
        pub mod common {
            pub mod v1 {
                include!(concat!(env!("OUT_DIR"), "/opentelemetry.proto.common.v1.rs"));
                // tonic::include_proto!("opentelemetry.proto.common.v1");
            }
        }
        pub mod profiles {
            pub mod v1development {
                include!(concat!(env!("OUT_DIR"), "/opentelemetry.proto.profiles.v1development.rs"));
            }
        }

        /// Generated types used in resources.
        pub mod resource {
            pub mod v1 {
                include!(concat!(env!("OUT_DIR"), "/opentelemetry.proto.resource.v1.rs"));
            }
        }

        // // the file_descriptor_set! macro is used to include the binary file descriptor set.
        // // It chooses the files according to the build.rs configuration.
        // pub const FILE_DESCRIPTOR_SET: &[u8] =
        //     tonic::include_file_descriptor_set!("logs_service_descriptor");//todo change to profiles_service_descriptor
    }
}

use std::collections::HashMap;
use std::fs::File;
use std::ops::Index;
use fallible_iterator::FallibleIterator;
use prost::Message;
use crate::symbfile::records::{Range, Record};
use crate::symbfile::read::Reader;
use crate::symbfile::proto::MessageType;

use opentelemetry::proto::profiles::v1development::Profile;
use opentelemetry::proto::collector::profiles::v1development::ExportProfilesServiceRequest;
use opentelemetry::proto::profiles::v1development::Sample;
use crate::test_symbolizer::opentelemetry::proto::common::v1::any_value;
use crate::test_symbolizer::opentelemetry::proto::common::v1::any_value::Value;
use crate::test_symbolizer::opentelemetry::proto::profiles::v1development::Mapping;

#[derive(Debug)]
struct SymbolInfo {
    function_name: String,
    source_file: Option<String>,
    call_line: Option<u32>,
}

/// Parse the symbfile using the Reader and build a mapping from ELF virtual addresses to symbols.
fn parse_symbfile(symbfile_path: &str) -> HashMap<u64, Record> {
    let file = File::open(symbfile_path).expect("Failed to open symbfile");
    let mut reader = Reader::new(file).expect("Failed to create symbfile reader");

    let mut address_to_symbol = HashMap::new();

    while let Some(record) = reader.next().expect("Error reading record from symbfile") {
        match record {
            Record::Range(ref range) => {
                address_to_symbol.insert(
                    range.elf_va,
                    record
                );
            }
            Record::ReturnPad(ref pad) => {
                // Handle ReturnPad if needed
                address_to_symbol.insert(
                    pad.elf_va,
                    record
                );
            }
        }
    }

    address_to_symbol
}

/// Parse the profile file.
fn parse_profile(profile_path: &str) -> Profile {
    let data = std::fs::read(profile_path).expect("Failed to read profile");
    Profile::decode(&*data).expect("Failed to decode profile")
}
fn parse_prequest(req_path: &str) ->ExportProfilesServiceRequest {
    let data = std::fs::read(req_path).expect("Failed to read equest");
    ExportProfilesServiceRequest::decode(&*data).expect("Failed to decode profile")
}
fn symbolize_request(req_path: &str, address_to_symbol: &HashMap<u64, Record>) {
    let req_files = std::fs::read_dir(req_path).expect("failed to read request directory")
        .filter_map(|entry| entry.ok())
        .filter(|entry|entry.path().is_file())
        .collect::<Vec<_>>();

    for x in &req_files {
        let path = x.path();
        let path  = path.to_str().unwrap();
        let req = parse_prequest(path);
        for res in req.resource_profiles{
            for scop in res.scope_profiles{
                for prof in scop.profiles{
                    symbolize_profile(&prof, address_to_symbol);
                }
            }
        }
    }
}



/// Perform symbolization of the profile using the address-to-symbol mapping.
// Perform symbolization of the profile using the address-to-symbol mapping.
fn symbolize_profile(profile: &Profile, address_to_symbol: &HashMap<u64, Record>) {
    // for x in &profile.mapping_table{
    //     // if (x.has_filenames) {
    //     //     println!("Filename: {:?}", (x.filename));
    //     // }
    //     // if (x.build_id != 0) {
    //     //     println!("Build ID: {:?}", (x.build_id));
    //     // }
    //
    // }
    //GO COde for finding the filename
    // for mi := 0; mi < profi.MappingTable().Len(); mi++ {
    //     mapping := profi.MappingTable().At(mi)
    //     filename := profi.StringTable().At(int(mapping.FilenameStrindex()))
    //     //gnuBuildID := profi.AttributeTable().At(mapping.AttributeIndices())//, "process.executable.build_id.gnu")
    //     println(filename)
    //     if strings.Contains(filename, "hello") {
    //         println("found profile for hello world ukremer. Id is:")
    ///ENd GO code




    for sample in &profile.sample{
        //println!("Sample: {:?}", sample);
        //find the mapping for the sample

        let attributes: Vec<_> = sample.attribute_indices.iter()
            .filter_map(|&index| profile.attribute_table.get(index as usize))
            .collect();
        for attr in &attributes {
            println!("Attributes ukremer: {:?}", attr);
            if (attr.key == "process.executable.path") {
                if let Some(any_value) = &attr.value {
                    match &any_value.value {
                        Some(Value::StringValue(val)) => {

                            println!("ukremer Key: {}, ukremer String Value: {}", attr.key, val);
                            if val.contains("hello") {
                                println!("found profile for hello world ukremer. Id is:")
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // let indices: Vec<usize> = sample.attribute_indices.iter().map(|&i| i as usize).collect();
        // let attributes: Vec<_> = indices
        //     .iter()
        //     .filter_map(|&index| profile.attribute_table.get(index))
        //     .collect();
        //
        // for attribute in attributes {
        //     println!("{:?}", attribute); // Replace with desired handling of attributes
        // }

        // Resolve the slice of locations using start_index and length
        let start_index = sample.locations_start_index as usize;
        let length = sample.locations_length as usize;
        //
        // Safely extract the locations slice
        let locations = profile
            .location_table
            .get(start_index..start_index + length)
            .unwrap_or(&[]);
        //
        // Process each location in the resolved slice
        for location in locations {
            let mut address = location.address;
            // Adjust the address using MappingTable
            if let Some(mapping) = profile.mapping_table.get(location.mapping_index.unwrap() as usize) {
                address += mapping.memory_start;
                let filename = profile.string_table.index(mapping.filename_strindex as usize);
                println!("Filename: {:?}", filename);
            }

            for (start, record) in address_to_symbol.iter() {
                if let Record::Range(range) = record {
                    if address >= range.elf_va && address < range.elf_va + range.length as u64{
                        println!("found address!!!");
                        let add = range.line_number_for_va(address);
                        println!("Address: 0x{:x}, Function: {}, File: {:?}, Line: {:?}",
                                 address,
                                 range.func,
                                 range.file,
                                 add
                        );
                        break;
                    }
                }
            }
            // Match the address with symbols from the symbfile
            // if let Some(symbol_info) = address_to_symbol.get(&address) {
            //    println!("found address!!!")
            //     // println!(
            //     //     "  Address: 0x{:x}, Function: {}, File: {:?}, Line: {:?}",
            //     //     address,
            //     //     symbol_info.function_name,
            //     //     symbol_info.source_file,
            //     //     symbol_info.call_line
            //     // );
            // } else {
            //     println!("  Address: 0x{:x}, Symbol: <unknown>", address);
            // }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbolize_request() {
        let symbfile_path = "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib-capi/output.symbfile";
        let req_path = "/home/ubuntu/git/opentelemetry-ebpf-profiler/test_data/requests";
        // Step 1: Parse the symbfile
        let address_to_symbol = parse_symbfile(symbfile_path);

        // Step 2: Parse the profile
        // Step 2: Parse each profile in the profiles folder
        symbolize_request(req_path, &address_to_symbol);
    }
    #[test]

    fn test_profiles() {
        let symbfile_path = "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib-capi/output.symbfile";
        let profiles_folder = "/home/ubuntu/git/opentelemetry-ebpf-profiler/profiles-proto";

        // Step 1: Parse the symbfile
        let address_to_symbol = parse_symbfile(symbfile_path);

        // Step 2: Parse the profile
        // Step 2: Parse each profile in the profiles folder
        for entry in std::fs::read_dir(profiles_folder).expect("Failed to read profiles folder") {
            let entry = entry.expect("Failed to read directory entry");
            let profile_path = entry.path();
            if profile_path.is_file() {
                // Step 3: Perform symbolization
                symbolize_profile_test(profile_path.to_str().expect("Invalid profile path"), &address_to_symbol);
            }
        }
        // Step 3: Perform symbolization
    }


    fn symbolize_profile_test(profile_path: &str, address_to_symbol: &HashMap<u64, Record>) {
        let symbfile_path = "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib-capi/output.symbfile";
        let profile_path = "/home/ubuntu/git/opentelemetry-ebpf-profiler/profile.proto";

        // Step 1: Parse the symbfile
        let address_to_symbol = parse_symbfile(symbfile_path);


        // Step 2: Parse the profile
        let profile = parse_profile(profile_path);

        // Step 3: Perform symbolization
        symbolize_profile(&profile, &address_to_symbol);
    }
}