use std::{env, path::PathBuf};

extern crate tonic_build;

// fn main() {
//     tonic_build::configure()
//         .out_dir("src/")
//         .compile(
//             &[
//                 "protobuf/change_events.proto",
//                 "protobuf/event_service.proto",
//             ],
//             &["protobuf/"],
//         )
//         .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(&["protobuf/change_events.proto"], &["protobuf"])
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));

    Ok(())
}
