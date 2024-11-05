use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(&["proto/grc20.proto"], &["proto/"])?;
    Ok(())
}
