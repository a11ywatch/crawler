fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/crawler.proto")?;
    tonic_build::compile_protos("proto/website.proto")?;

    Ok(())
}
