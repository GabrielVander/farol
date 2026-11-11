//! The testing API for farol: fakes for the gateways, fixtures, and a scenario
//! that drives the use cases. Tests depend on it; production code never does.

// A broken fixture should fail the test that uses it, so panicking is the contract.
#![allow(clippy::missing_panics_doc)]

pub mod fakes;
pub mod fixtures;
