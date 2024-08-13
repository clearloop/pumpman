//! Replika service
#![allow(async_fn_in_trait)]

pub use {cli::Opt, config::Config};

pub mod api;
mod cli;
mod config;
mod context;
mod model;
mod schema;
mod service;
pub mod sol;
pub mod telegram;
mod utils;
