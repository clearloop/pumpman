//! Context related methods

use crate::{
    context::Context,
    schema::{takeovers, users},
};
use anyhow::Result;
use diesel::prelude::*;

impl Context {
    /// Check if the user has enough credits
    pub fn eligible(&self, uid: u64) -> Result<bool> {
        let postgres = &mut self.postgres()?;
        let uid = uid.to_string();

        let count = takeovers::table
            .filter(takeovers::admin.eq(&uid))
            .count()
            .execute(postgres)? as i64;

        let credits = users::table
            .select(users::credits)
            .filter(users::tgid.eq(uid))
            .first(postgres)
            .optional()?
            .unwrap_or(1);

        return Ok(credits > count);
    }
}
