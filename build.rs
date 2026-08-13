use std::{env, error::Error};

use vergen_gix::{Build, Emitter, Gix};

fn main() -> Result<(), Box<dyn Error>> {
    let mut v = Emitter::default();

    v.add_instructions(&Build::builder().build_date(true).build())?;

    match env::var_os("JB_NO_GIT") {
        Some(_) => {
            println!("cargo::rustc-env=VERGEN_GIT_SHA=no-git");
        }
        None => {
            v.add_instructions(&Gix::builder().sha(true).build())?;
        }
    }

    v.emit()?;

    Ok(())
}
