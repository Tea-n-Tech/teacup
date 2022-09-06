extern crate tonic_build;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=migrations");

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(&["protobuf/change_events.proto"], &["protobuf"])
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));

    Ok(())
}
