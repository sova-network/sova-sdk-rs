pub mod auth {
    tonic::include_proto!("auth");
}

pub mod searcher {
    tonic::include_proto!("searcher");
}

pub mod dto {
    tonic::include_proto!("dto");
}

pub mod block_engine {
    tonic::include_proto!("block_engine");
}
