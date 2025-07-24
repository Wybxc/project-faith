fn main() {
    tonic_build::configure()
        .compile_protos(&["proto/auth.v1.proto", "proto/game.v1.proto"], &["proto"])
        .unwrap();
}
