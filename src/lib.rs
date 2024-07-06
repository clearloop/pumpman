//! Replika service
#![allow(unused)]

pub use {cli::Opt, config::Config, context::redis};

mod cli;
mod config;
mod context;
mod model;
mod schema;
mod sol;
pub mod telegram;
mod utils;
