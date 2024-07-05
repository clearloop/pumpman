//! Replika service
#![allow(unused)]

pub use {cli::Opt, client::Client, config::Config};

mod cli;
mod client;
mod config;
mod context;
mod model;
mod schema;
mod sol;
mod telegram;
mod utils;
