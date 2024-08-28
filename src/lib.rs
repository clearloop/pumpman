//! Replika service
#![allow(async_fn_in_trait)]

pub use {cli::Opt, config::Config, context::Context};

pub mod api;
mod cli;
pub mod config;
pub mod context;
pub mod emoji;
mod model;
mod schema;
pub mod service;
pub mod sol;
pub mod telegram;
mod utils;
