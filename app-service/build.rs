/**
 * Reference:
 * - https://github.com/hyperium/tonic/issues/1020
 */

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("./proto/authentication.proto")?;
    println!("cargo:rerun-if-changed=proto/authentication.proto");
    Ok(())
}
