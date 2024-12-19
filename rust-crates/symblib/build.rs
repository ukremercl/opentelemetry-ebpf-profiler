// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

static PROTO: &str = "../symb-proto/symbfile.proto";

fn main() -> Result<(), Box<dyn std::error::Error>> {

    println!("cargo:rerun-if-changed={PROTO}");
    println!("cargo:rerun-if-changed=/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib/target/proto/opentelemetry/proto/profiles/v1development/profiles.proto");
    // println!("cargo:rerun-if-changed=/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib/target/proto/opentelemetry/proto/profiles/v1development/pprofextended.proto");
    let mut config = prost_build::Config::new();
    // Add the experimental option for proto3 optional fields
    config.protoc_arg("--experimental_allow_proto3_optional");

    config.compile_protos(
        &[PROTO,
            "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib/target/proto/opentelemetry/proto/common/v1/common.proto",
            "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib/target/proto/opentelemetry/proto/resource/v1/resource.proto",
            "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib/target/proto/opentelemetry/proto/profiles/v1development/profiles.proto",
            "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib/target/proto/opentelemetry/proto/collector/profiles/v1development/profiles_service.proto"],
        &["../symb-proto","/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib/target/proto"],
    )?;

    Ok(())
    // Ok(prost_build::compile_protos(
    //     &[PROTO,
    //         "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib/target/proto/opentelemetry/proto/common/v1/common.proto",
    //         "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib/target/proto/opentelemetry/proto/resource/v1/resource.proto",
    //         "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib/target/proto/opentelemetry/proto/profiles/v1development/profiles.proto"],
    //     // "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib/target/proto/opentelemetry/proto/profiles/v1development/pprofextended.proto"],
    //     &["../symb-proto","/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib/target/proto"],
    // )?)

}
