fn main() {
    tonic_build::compile_protos("grpc/proto/auth.proto").unwrap();
    tonic_build::compile_protos("grpc/proto/searcher.proto").unwrap();
    tonic_build::compile_protos("grpc/proto/dto.proto").unwrap();
    tonic_build::compile_protos("grpc/proto/block_engine.proto").unwrap();
}
