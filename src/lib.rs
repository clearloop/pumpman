//! Replika service

pub use {cli::Opt, config::Config};

mod api;
mod cli;
mod config;
mod context;
mod model;
mod schema;
mod service;
pub mod sol;
pub mod telegram;
mod utils;
