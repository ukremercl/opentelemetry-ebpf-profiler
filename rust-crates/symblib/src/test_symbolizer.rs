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
        //         pub mod v1experimental {
        //             include!(concat!(env!("OUT_DIR"), "/opentelemetry.proto.profiles.v1experimental.rs"));
        //
        //             // tonic::include_proto!("opentelemetry.proto.collector.profiles.v1experimental");
        //             // tonic::include_proto!("opentelemetry.proto.collector.profiles.v1experimental.ProfilesService");
        //         }
        //     }
        // }

        /// Common types used across all signals
        pub mod common {
            pub mod v1 {
                include!(concat!(env!("OUT_DIR"), "/opentelemetry.proto.common.v1.rs"));
                // tonic::include_proto!("opentelemetry.proto.common.v1");
            }
        }
        pub mod profiles {
            pub mod v1experimental {
                include!(concat!(env!("OUT_DIR"), "/opentelemetry.proto.profiles.v1experimental.rs"));
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
use fallible_iterator::FallibleIterator;
use prost::Message;
use crate::symbfile::records::{Range, Record};
use crate::symbfile::read::Reader;
use crate::symbfile::proto::MessageType;

use opentelemetry::proto::profiles::v1experimental::Profile;
use opentelemetry::proto::profiles::v1experimental::Sample;
use crate::test_symbolizer::opentelemetry::proto::profiles::v1experimental::Mapping;

#[derive(Debug)]
struct SymbolInfo {
    function_name: String,
    source_file: Option<String>,
    call_line: Option<u32>,
}

/// Parse the symbfile using the Reader and build a mapping from ELF virtual addresses to symbols.
fn parse_symbfile(symbfile_path: &str) -> HashMap<u64, SymbolInfo> {
    let file = File::open(symbfile_path).expect("Failed to open symbfile");
    let mut reader = Reader::new(file).expect("Failed to create symbfile reader");

    let mut address_to_symbol = HashMap::new();

    while let Some(record) = reader.next().expect("Error reading record from symbfile") {
        match record {
            Record::Range(range) => {
                address_to_symbol.insert(
                    range.elf_va,
                    SymbolInfo {
                        function_name: range.func,
                        source_file: range.file,
                        call_line: range.call_line,
                    },
                );
            }
            Record::ReturnPad(_) => {
                // Handle ReturnPad if needed
                continue;
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

/// Perform symbolization of the profile using the address-to-symbol mapping.
// Perform symbolization of the profile using the address-to-symbol mapping.
fn symbolize_profile(profile: &Profile, address_to_symbol: &HashMap<u64, SymbolInfo>) {
    for x in &profile.mapping {
        if (x.has_filenames) {
            println!("Filename: {:?}", (x.filename));
        }
        if (x.build_id != 0) {
            println!("Build ID: {:?}", (x.build_id));
        }
    }
    for (sample_index, sample) in profile.sample.iter().enumerate() {
        println!("Sample {}:", sample_index);

        // Resolve the slice of locations using start_index and length
        let start_index = sample.locations_start_index as usize;
        let length = sample.locations_length as usize;

        // Safely extract the locations slice
        let locations = profile
            .location
            .get(start_index..start_index + length)
            .unwrap_or(&[]);

        // Process each location in the resolved slice
        for location in locations {
            let address = location.address;

            // Match the address with symbols from the symbfile
            if let Some(symbol_info) = address_to_symbol.get(&address) {
                println!(
                    "  Address: 0x{:x}, Function: {}, File: {:?}, Line: {:?}",
                    address,
                    symbol_info.function_name,
                    symbol_info.source_file,
                    symbol_info.call_line
                );
            } else {
                println!("  Address: 0x{:x}, Symbol: <unknown>", address);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn symbolize_profile_test(profile_path: &str, address_to_symbol: &HashMap<u64, SymbolInfo>) {
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