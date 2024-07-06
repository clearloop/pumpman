//! Replika service
#![allow(unused)]

pub use {cli::Opt, config::Config};

mod cli;
mod config;
mod context;
mod model;
mod schema;
mod sol;
mod telegram;
mod utils;
