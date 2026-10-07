#![allow(missing_docs)]

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let protos = [
        std::path::PathBuf::from("proto/acorn/service/service.proto"),
        std::path::PathBuf::from("proto/grpc/health/v1/health.proto"),
    ];
    protos.iter().for_each(|proto| println!("cargo:rerun-if-changed={}", proto.display()));
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
                .type_attribute(".", "#[allow(missing_docs)]")
                .compile_protos(&protos, &[std::path::PathBuf::from("proto"), include])
                .map_err(Box::<dyn core::error::Error>::from)
        })
}
