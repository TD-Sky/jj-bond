use std::error::Error;

use vergen_gix::{Build, Emitter, Gix};

fn main() -> Result<(), Box<dyn Error>> {
    Emitter::default()
        .add_instructions(&Build::builder().build_date(true).build())?
        .add_instructions(&Gix::builder().sha(true).build())?
        .emit()?;

    Ok(())
}
