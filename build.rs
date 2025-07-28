fn main() {
    tonic_build::configure()
        .build_client(false)
        .compile_protos(
            &[
                "proto/auth.v1.proto",
                "proto/game.v1.proto",
                "proto/card.v1.proto",
            ],
            &["proto"],
        )
        .unwrap();
}
