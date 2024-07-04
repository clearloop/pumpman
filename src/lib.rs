//! Replika service

pub use {cli::Opt, client::Client, config::Config, context::Redis};

mod cli;
mod client;
mod config;
mod context;
mod sol;
mod utils;
