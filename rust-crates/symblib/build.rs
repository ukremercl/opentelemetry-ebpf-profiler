// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

static PROTO: &str = "../symb-proto/symbfile.proto";

fn main() -> Result<(), Box<dyn std::error::Error>> {

    println!("cargo:rerun-if-changed={PROTO}");
    println!("cargo:rerun-if-changed=/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib/target/proto/opentelemetry/proto/profiles/v1experimental/profiles.proto");
    println!("cargo:rerun-if-changed=/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib/target/proto/opentelemetry/proto/profiles/v1experimental/pprofextended.proto");

    Ok(prost_build::compile_protos(
        &[PROTO,
            "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib/target/proto/opentelemetry/proto/common/v1/common.proto",
            "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib/target/proto/opentelemetry/proto/resource/v1/resource.proto",
            "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib/target/proto/opentelemetry/proto/profiles/v1experimental/profiles.proto",
        "/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib/target/proto/opentelemetry/proto/profiles/v1experimental/pprofextended.proto"],
        &["../symb-proto","/home/ubuntu/git/opentelemetry-ebpf-profiler/rust-crates/symblib/target/proto"],
    )?)

}
