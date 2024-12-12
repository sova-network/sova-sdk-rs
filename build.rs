fn main() {
    let protbuf_files = [
        "auth.proto",
        "searcher.proto",
        "dto.proto",
        "block_engine.proto",
    ];

    tonic_build::configure()
        // The `optional` keyword in the message requires compiling the .proto file with
        // the `--experimental_allow_proto3_optional` flag (see https://github.com/hyperium/tonic/issues/627)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(&protbuf_files, &["grpc/proto"])
        .expect("Failed to compile protobuf files");
}
