//! Context related methods

use crate::{
    context::Context,
    schema::{takeovers, users},
};
use anyhow::Result;
use diesel::prelude::*;

impl Context {
    /// Check if the user has enough credits
    pub fn eligible(&self, uid: &str) -> Result<bool> {
        let postgres = &mut self.postgres()?;

        let count = takeovers::table
            .filter(takeovers::admin.eq(&uid))
            .count()
            .get_result(postgres)?;

        let credits = users::table
            .select(users::credits)
            .filter(users::tgid.eq(&uid))
            .first(postgres)
            .optional()?
            .unwrap_or(1);

        return Ok(credits > count);
    }
}
