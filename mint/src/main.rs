//! Mints the definitions of the Koinon Commons' models into `.akev` files beside the structs
//! that declare them, and checks those files.

use std::{path::Path, process::ExitCode};

use koinon_commons::namespaces::NAMESPACES;

fn main() -> ExitCode {
    let mint = Path::new(env!("CARGO_MANIFEST_DIR"));
    koinon_mint::run(NAMESPACES, &mint.parent().expect("the mint crate sits in the workspace").join("src"))
}
