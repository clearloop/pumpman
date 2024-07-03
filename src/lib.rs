//! Replika service

pub use {cli::Opt, client::Client};

mod cli;
mod client;
mod config;
mod context;
