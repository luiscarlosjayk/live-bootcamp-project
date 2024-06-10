/**
 * References:
 * - https://github.com/hyperium/tonic/issues/1020
 * - https://docs.rs/sqlx/latest/sqlx/macro.migrate.html#triggering-recompilation-on-migration-changes
 */

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tonic_build::compile_protos("./proto/authentication.proto")?;
    // println!("cargo:rerun-if-changed=proto/authentication.proto");
    println!("cargo:rerun-if-changed=src/main.rs");
    Ok(())
}
