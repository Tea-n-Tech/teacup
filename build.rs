extern crate prost_build;

fn main() {
    std::env::set_var("OUT_DIR", "src/");
    let mut config = prost_build::Config::new();
    config.out_dir("src/");

    prost_build::compile_protos(&["protobuf/change_events.proto"], &["protobuf/"]).unwrap();
}
