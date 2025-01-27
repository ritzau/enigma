fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(tonic_build::compile_protos(
        "../../../api/proto/profiles.proto",
    )?)
}
