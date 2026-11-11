use std::ffi::OsString;

use clap::Parser;

use crate::{
    adapters::controllers::records_controller::{Infrastructure, Termination, handle},
    infrastructure::command_line::Cli,
};

/// Runs farol for `arguments`, which start with the program name. Fails only
/// when the command line itself is wrong, or asks for help or the version.
pub fn run<T: Into<OsString> + Clone>(
    arguments: impl IntoIterator<Item = T>,
    infrastructure: &impl Infrastructure,
) -> Result<Termination, clap::Error> {
    let cli = Cli::try_parse_from(arguments)?;
    Ok(handle(&cli.into_request(), infrastructure))
}
