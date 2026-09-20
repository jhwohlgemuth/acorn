#![allow(missing_docs)]

#[cfg(feature = "grpc")]
fn generate_grpc() -> Result<(), Box<dyn core::error::Error>> {
    let proto = std::path::PathBuf::from("proto/acorn/rpc/v1/rpc.proto");
    protoc_bin_vendored::protoc_bin_path()
        .map_err(Box::<dyn core::error::Error>::from)
        .and_then(|protoc| {
            protoc_bin_vendored::include_path()
                .map_err(Box::<dyn core::error::Error>::from)
                .map(|include| (protoc, include))
        })
        .and_then(|(protoc, include)| {
            std::env::set_var("PROTOC", protoc);
            tonic_prost_build::configure()
                .compile_protos(&[proto], &[std::path::PathBuf::from("proto"), include])
                .map_err(Box::<dyn core::error::Error>::from)
        })
}

fn main() {
    println!("cargo:rerun-if-changed=proto/acorn/rpc/v1/rpc.proto");
    #[cfg(feature = "grpc")]
    if let Err(why) = generate_grpc() {
        eprintln!("Failed to generate ACORN gRPC bindings: {why}");
        std::process::exit(1);
    }
}
