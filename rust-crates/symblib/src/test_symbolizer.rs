#![allow(missing_docs)]

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
use std::io;
use std::io::{Seek, SeekFrom, Write};
use std::ops::Index;
use std::path::Path;
use fallible_iterator::FallibleIterator;
use prost::Message;
use crate::symbfile::records::{Range, Record};
use crate::symbfile::read::Reader;
use crate::symbfile::proto::MessageType;

use opentelemetry::proto::profiles::v1development::Profile;
use opentelemetry::proto::collector::profiles::v1development::ExportProfilesServiceRequest;
use opentelemetry::proto::profiles::v1development::Sample;
use crate::retpads::create_retpad_symbfile;
use crate::symbfile;
use crate::symbfile::ReturnPad;
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
fn parse_symbfile_to_vec(symbfile_path_vec: Vec<&str>) -> Vec<Record> {
    let mut range_recs = Vec::<Record>::new();
    symbfile_path_vec.iter().for_each(|symbfile_path| {
        let file = File::open(symbfile_path).expect("Failed to open symbfile");
        let mut reader = Reader::new(file).expect("Failed to create symbfile reader");

        while let Some(record) = reader.next().expect("Error reading record from symbfile") {
            match record {
                Record::Range(ref range) => {
                    range_recs.push(record);
                }
                Record::ReturnPad(ref pad) => {
                    // Handle ReturnPad if needed
                    range_recs.push(record);
                }
            }
        }
    });
    range_recs
}


// 1780-2000, 2004-2044,2044-2144, 2076-2096,
/// Parse the profile file.
fn parse_profile(profile_path: &str) -> Profile {
    let data = std::fs::read(profile_path).expect(&format!("Failed to read profile: {}", profile_path));
    Profile::decode(&*data).expect(&format!("Failed to decode profile: {}", profile_path))
}
fn parse_prequest(req_path: &str) ->ExportProfilesServiceRequest {
    let data = std::fs::read(req_path).expect("Failed to read equest");
    ExportProfilesServiceRequest::decode(&*data).expect(&format!("Failed to decode profile: {}", req_path))
}

fn get_return_pad_vec(exec_path: &str, range_symbfile: &str) -> Vec<ReturnPad> {
    // let exec_path = testdata("inline-no-tco");
    // let range_symbfile = File::open(testdata("inline-no-tco.ranges.symbfile")).unwrap();
    let mut retpad_symbfile = tempfile::tempfile().unwrap();
    let rng_file = File::open(range_symbfile).expect("Failed to open range symbfile");


    create_retpad_symbfile(Path::new(&exec_path), rng_file, &mut retpad_symbfile).unwrap();
    retpad_symbfile.seek(SeekFrom::Start(0)).unwrap();

    let mut reader = symbfile::Reader::new(retpad_symbfile).unwrap();

    // The message order in return pad symbfiles is undefined: read all and sort.
    let mut records = Vec::<symbfile::ReturnPad>::new();
    while let Some(msg) = reader.read().unwrap() {
        records.push(match msg {
            symbfile::Record::ReturnPad(pad) => pad,
            _ => panic!("unexpected record type"),
        });
    }
    records.sort_unstable_by_key(|x| x.elf_va);
    records
}

fn symbolize_request(req_path: &str, symbfile_path: Vec<&str>, pads_option : Option<&Vec<ReturnPad>>, pid_filter: ::std::option::Option<i64>) {
    let range_vec = parse_symbfile_to_vec(symbfile_path);

    let req_files = std::fs::read_dir(req_path).expect(&format!("failed to read request directory {}",req_path))
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
                    symbolize_profile(&prof, &range_vec, pads_option, pid_filter);
                }
            }
        }
    }
}



/// Perform symbolization of the profile using the address-to-symbol mapping.
// Perform symbolization of the profile using the address-to-symbol mapping.
fn symbolize_profile(profile: &Profile, range_rec: &Vec<Record>, pads_option: Option<&Vec<ReturnPad>>, pid_filter: ::std::option::Option<i64>) {

    // Iterate over each sample in the profile
    for sample in &profile.sample{
        //println!("Sample: {:?}", sample);
        //find the mapping for the sample size=11

        let attributes: Vec<_> = sample.attribute_indices.iter()
            .filter_map(|&index| profile.attribute_table.get(index as usize))
            .collect();
        //Init pid and executable path
        let mut pid = 0;
        let mut executable_path = "";
        for attr in &attributes {
            println!("Attributes ukremer: {:?}", attr);
            if attr.key == "process.pid" && pid_filter.is_some() {
                if let Some(any_value) = &attr.value {
                    match &any_value.value {
                        Some(Value::IntValue(val)) => {
                            pid = *val;
                            println!("PID: {}", pid);
                            println!("thread name: {:?}", attr.key);

                        }
                        _ => {}
                    }
                }
            }

            if (attr.key == "process.executable.path") {
                if let Some(any_value) = &attr.value {
                    match &any_value.value {
                        Some(Value::StringValue(val)) => {

                            println!("ukremer Key: {}, ukremer String Value: {}", attr.key, val);
                            if val.contains("hello") {
                                println!("found profile for hello world ukremer. Id is:")
                            }
                            executable_path = val;
                        }
                        _ => {}
                    }
                }
            }
        }
        if pid_filter.is_some() && pid != pid_filter.unwrap() {
            continue;
        }
        println!("PID: {}", pid);
        println!("Executable path: {}", executable_path);

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
            let locc = location.clone();
            let mut address = location.address;
            let mut addMinus = location.address;
            // Adjust the address using MappingTable
            let mut filename = "";
            let mut buildid = "";
            if let Some(mapping) = profile.mapping_table.get(location.mapping_index.unwrap() as usize) {
                address += mapping.memory_start;
                address += mapping.file_offset;
                filename = profile.string_table.index(mapping.filename_strindex as usize);
                println!("Filename: {:?}", filename);
                addMinus = address - mapping.memory_start - mapping.file_offset;

            }else {
                continue;
            }

            println!("CHecking address: {:?}", address);

            if !filename.contains("hello"){
               continue;
            }
            for (record) in range_rec.iter() {
                if let Record::Range(range) = record {
                    let rrange = range.clone();
                    if address >= range.elf_va && address < range.elf_va + range.length as u64{
                        println!("found address!!!");
                        println!("Rrange: {:?} ", rrange);
                        let line = range.line_number_for_va(address);
                        println!("Address: 0x{:x}, Function: {}, File: {:?}, Line: {:?}",
                                 address,
                                 range.func,
                                 range.file,
                                 line
                        );
                    }
                }
            }

            if let Some(pads) = pads_option{
                for pad in pads{
                    let pp = &pad;
                    //println!("pp: {:?}", pp);
                    if address == pad.elf_va{
                        println!("found pad for address{}",address);
                        for x in pad.entries.iter() {
                            let f = &x.func;
                            println!("pad Func: {:?}", f);
                            let n = &x;
                            println!("Pad: {:?}", n);

                        }
                        println!("Sample ukremer found: {:?}", sample);
                        // io::stdout().flush().unwrap();
                        // io::stderr().flush().unwrap();
                        // break;
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
    fn test_single_func_symbolize_request() {
        let symbfile_path = "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib-capi/output.symbfile";
        let req_path = "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/test_data/requests";
        // Step 1: Parse the symbfile

        // Step 2: Parse the profile
        // Step 2: Parse each profile in the profiles folder
        let symbfile_path_vec = vec![symbfile_path];
        symbolize_request(req_path, symbfile_path_vec, None, None);
    }

    #[test]
    fn test_file_with_single_func(){
        let sym_path = "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib-capi/hello_with_functions.symbfile";
        let profiles_folder = "/home/ubuntu/git/opentelemetry-ebpf-profiler/profiles-proto";
        test_profiles(sym_path, profiles_folder);
    }


    fn test_profiles(symbfile_path: &str, profiles_folder: &str) {
        // let symbfile_path = "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib-capi/output.symbfile";
        // let profiles_folder = "/home/ubuntu/git/opentelemetry-ebpf-profiler/profiles-proto";

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



    #[test]
    fn test_parse_symbfile2(){
        let parsed = parse_symbfile("/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib-capi/output_mul_inline.symbfile");
    }
    #[test]
    fn test_profile_symbolize() {
        let symbfile_path = "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib-capi/output.symbfile";
        let profile_path = "/home/ubuntu/git/opentelemetry-ebpf-profiler/profile.proto";

        // Step 1: Parse the symbfile
        let address_to_symbol = parse_symbfile(symbfile_path);

        // Step 2: Parse the profile
        let profile = parse_profile(profile_path);

        // Step 3: Perform symbolization
        let sym_vec = vec![symbfile_path];
        symbolize_profile(&profile, &parse_symbfile_to_vec(sym_vec), None, None);
    }
    fn symbolize_profile_test(profile_path: &str, address_to_symbol: &HashMap<u64, Record>) {
        let symbfile_path = "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib-capi/output.symbfile";
        let profile_path = "/home/ubuntu/git/opentelemetry-ebpf-profiler/profile.proto";

        // Step 1: Parse the symbfile
        // let address_to_symbol = parse_symbfile(symbfile_path);


        // Step 2: Parse the profile
        let profile = parse_profile(profile_path);

        // Step 3: Perform symbolization
        let sym_vec = vec![symbfile_path];

        symbolize_profile(&profile, &parse_symbfile_to_vec(sym_vec) , None, None);
    }

    #[test]
    fn test_file_with_multiple_funcs(){
        let sym_path = "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib-capi/output_mul_only_inline.symbfile";//"/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib-capi/output_mul_inline.symbfile";
        let req_folder = "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/test_data/req3-single-inl";//"/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/test_data/req2";
        let rec_vec =
            get_return_pad_vec("/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/test/hello_mul_only_inline",
                               "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib-capi/output_mul_only_inline.symbfile",
            );

        let sym_vec = vec![sym_path];

        symbolize_request(req_folder, sym_vec, Some(&rec_vec), Some(1177588));
    }
    #[test]
    fn test_file_with_extern_libs(){
        let sym_path = "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/test_data/req_with_lib/hello_with_libs.symbfile";

        let req_folder = "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/test_data/req_with_lib/reqs/";//"/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/test_data/req2";
        let rec_vec =
            get_return_pad_vec("/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/test/hello_with_libs",
                               sym_path,
            );
        let sym_vec = vec![sym_path];
        symbolize_request(req_folder, sym_vec, Some(&rec_vec), None);
    }

    #[test]
    fn test_file_with_extern_libs_only_lib(){
        let sym_path = "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/test_data/req_with_lib/libcrypto2.symbfile";

        let req_folder = "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/test_data/req_with_lib/reqs/";//"/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/test_data/req2";
        let rec_vec =
            get_return_pad_vec("/home/ubuntu/.cache/debuginfod_client/e1ccd314d3d3e596c8d9b70001c917e9c5292c33/debuginfo",
                               sym_path,
            );
        let sym_vec = vec![sym_path];

        symbolize_request(req_folder, sym_vec, Some(&rec_vec), None);
    }

    #[test]
    fn test_file_with_extern_libs_inline_stl(){
        let lib_sym_path = "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/test_data/req_with_lib/libcrypto2.symbfile";
        let exec_sym_path = "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/test_data/req6_with_lib_and_inline/hello2_with_libs_stk_in.symbfile";
        let req_folder =    "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/test_data/req6_with_lib_and_inline/profiles";
        let exec_path = "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/test/hello2_with_libs_stk_inline";

        let sym_vec = vec![ exec_sym_path, lib_sym_path];//lib_sym_path,
        let pad_vec_exec =
            get_return_pad_vec(
                               exec_path,
                               exec_sym_path
            );


        let rec_vec =
            get_return_pad_vec("/home/ubuntu/.cache/debuginfod_client/e1ccd314d3d3e596c8d9b70001c917e9c5292c33/debuginfo",
                               lib_sym_path,
            );
        symbolize_request(req_folder, sym_vec, Some(&rec_vec), Some(1804951));
    }


    #[test]
    fn test_parse_pads(){
        let rec_vec =
            get_return_pad_vec("/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/test/hello_mul_only_inline",
            "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib-capi/output_mul_only_inline.symbfile",
        );
        for x in rec_vec{
            println!("{:?}", x);
        }

        // ReturnPad { elf_va: 1775, entries: [ReturnPadEntry { func: "_start", file: None, line: None }] }
        // ReturnPad { elf_va: 1779, entries: [ReturnPadEntry { func: "_start", file: None, line: None }] }
        // ReturnPad { elf_va: 2031, entries: [ReturnPadEntry { func: "print_hello_world", file: Some("/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/test/hello.c"), line: Some(4) }] }
        // ReturnPad { elf_va: 2083, entries: [ReturnPadEntry { func: "main", file: Some("/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/test/hello.c"), line: Some(13) }, ReturnPadEntry { func: "foo", file: Some("/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/test/hello.c"), line: Some(7) }] }
        // ReturnPad { elf_va: 2095, entries: [ReturnPadEntry { func: "main", file: Some("/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/test/hello.c"), line: Some(15) }] }
        // ReturnPad { elf_va: 2119, entries: [ReturnPadEntry { func: "main", file: Some("/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/test/hello.c"), line: Some(16) }, ReturnPadEntry { func: "foo", file: Some("/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/test/hello.c"), line: Some(7) }] }
        // ReturnPad { elf_va: 2131, entries: [ReturnPadEntry { func: "main", file: Some("/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/test/hello.c"), line: Some(17) }] }
    }
}