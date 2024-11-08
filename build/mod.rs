// build/mod.rs
use std::env;
use std::path::PathBuf;

mod bindings;
mod error;
mod library;
mod stub;

use error::BuildError;
pub use error::Result;

pub fn main() -> Result<()> {
    let rss_path = library::get_rss_path()?;
    let lib_path = if cfg!(feature = "stub_library") {
        let out_dir = PathBuf::from(env::var("OUT_DIR").map_err(BuildError::EnvVar)?);
        stub::generate_stubs(&rss_path, &out_dir)?;
        out_dir
    } else {
        library::discover_library()?
    };

    // Setup linking and generate bindings
    library::setup_linking(&lib_path)?;
    bindings::generate_bindings(&rss_path)?;

    // Always rerun if these change
    println!("cargo:rerun-if-changed=build/");
    println!("cargo:rerun-if-changed=c_src/");
    println!("cargo:rerun-if-env-changed=ACC_RSS_LIBS");

    Ok(())
}
