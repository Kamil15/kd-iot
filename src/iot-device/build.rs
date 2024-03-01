use std::io::Result;
fn main() -> Result<()> {
    prost_build::compile_protos(&["src/proto/proto_broker_msgs.proto"], &["src/"])?;
    
    Ok(())
}