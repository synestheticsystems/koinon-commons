//! Mints the definitions of a crate's Koinon models into `.akev` files beside the structs that
//! declare them, and checks those files, so that every node that seeds the crate binds its
//! models to the same definitions. It does for any crate that lists its namespaces as the
//! Ankurah Commons lists its own, as its `Namespace`, what the Commons' mint does for the
//! Commons' models, through the Commons' `registration` and `akev`. It mints models only: a
//! namespace that lists kinds of edge is refused, since the Commons' own mint mints those.
//!
//! The declarations are compiled in, so each crate the tool mints has a binary of its own that
//! calls `run` with its list. This crate's binary mints the Koinon Commons.
//!
//! `mint` reads every kept file under the namespaces' parent directory, and for each model of
//! each namespace it takes from the struct's declaration the events that registering it writes:
//! a definition of the model, and of each property with the link that gives the model the
//! property. Each event keeps the nonce and timestamp of an event of the model's kept file whose
//! operations are exactly the same, and otherwise draws a fresh nonce and the current time, so
//! that minting a kept mint again changes nothing unless a declaration changed. It writes the
//! events into `<model>.akev` beside the `<model>.rs` that declares the model, and gives the
//! struct its model's id, `#[model(id = "...")]`.
//!
//! `verify` checks that every listed model has its file beside its struct and no other `.akev`
//! file is there, that every file reads, that each states exactly the events its model's
//! declaration needs, and that the struct declares the id the file states. It does not show who
//! made the events, since nothing is signed: what vouches for a kept mint is the repository it is
//! committed to.

#[cfg(test)]
mod copies;
mod declaring;
mod minting;
mod placement;
mod verifying;
mod writing;

use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

use ankurah_commons::namespaces::Namespace;
use clap::Parser;
use thiserror::Error;

use crate::{minting::MintError, writing::WriteError};

pub use verifying::{verify, Failure, Verified, VerifyError};

#[derive(Parser)]
#[command(about = "Mints the definitions of a crate's Koinon models into .akev files, and checks them")]
enum Command {
    /// Mint the definitions of every model of every namespace into `.akev` files beside the
    /// structs that declare them. An entity whose event the kept mint there holds keeps that
    /// event's nonce and timestamp.
    Mint {
        /// The namespaces' parent directory, when not the crate's `src/`.
        #[arg(long)]
        src: Option<PathBuf>,
    },
    /// Check the `.akev` files a mint keeps.
    Verify {
        /// The namespaces' parent directory, when not the crate's `src/`.
        #[arg(long)]
        src: Option<PathBuf>,
    },
}

/// Why a command failed.
#[derive(Debug, Error)]
enum CommandError {
    #[error("minting failed: {0}")]
    Mint(#[from] MintError),
    #[error("writing failed: {0}")]
    Write(#[from] WriteError),
    #[error("verification failed: {0}")]
    Verify(#[from] VerifyError),
}

/// Runs the command line for the crate whose namespaces are `namespaces`, and whose namespaces'
/// parent directory is `sources` unless `--src` names another.
pub fn run(namespaces: &[Namespace], sources: &Path) -> ExitCode {
    let result = match Command::parse() {
        Command::Mint { src } => mint_command(src.as_deref().unwrap_or(sources), namespaces),
        Command::Verify { src } => verify_command(src.as_deref().unwrap_or(sources), namespaces),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn mint_command(src: &Path, namespaces: &[Namespace]) -> Result<(), CommandError> {
    let minted = minting::mint(src, namespaces)?;
    writing::write(&minted)?;

    let properties: usize = minted.iter().map(|file| file.descriptor.properties.len()).sum();
    let drawn: usize = minted.iter().map(|file| file.drawn.len()).sum();
    println!(
        "Minted {} models in {} namespaces, with {properties} properties and their links, into {}; {drawn} events drawn anew",
        minted.len(),
        namespaces.len(),
        src.display()
    );
    Ok(())
}

fn verify_command(src: &Path, namespaces: &[Namespace]) -> Result<(), CommandError> {
    let verified = verify(src, namespaces)?;
    println!("Verified {} files, reading {} creation events", verified.files, verified.events);
    Ok(())
}
