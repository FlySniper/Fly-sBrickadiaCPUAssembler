mod op_codes;
mod patch_builder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    op_codes::write_brz()?;
    Ok(())
}
