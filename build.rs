// extern crate prost_build;

// fn main() {
//     std::env::set_var("OUT_DIR", "src/");
//     let mut config = prost_build::Config::new();
//     config.out_dir("src/");

//     prost_build::compile_protos(
//         &[
//             "protobuf/change_events.proto",
//             "protobuf/event_service.proto",
//         ],
//         &["protobuf/"],
//     )
//     .unwrap();
// }

extern crate tonic_build;

fn main() {
    tonic_build::configure()
        .out_dir("src/")
        .compile(
            &[
                "protobuf/change_events.proto",
                "protobuf/event_service.proto",
            ],
            &["protobuf/"],
        )
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
    // tonic_build::compile_protos("protobuf/change_events.proto")
    //     .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
}
