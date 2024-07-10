//! tasks queue

use crate::{context::TaskCache, sol::pump::events};
use anyhow::Result;
use redis::{Commands, Connection};
use std::collections::HashSet;

use super::ArcMutex;

/// Task queue
#[derive(Clone, Default)]
pub struct Task {
    /// Queue for checking soldout
    pub soldout: ArcMutex<HashSet<String>>,

    /// Queue for checking holder changes
    pub holders: ArcMutex<HashSet<(String, u8)>>,
    // /// Queue for checking big deals
    // pub whale: HashMap<String, events::TradeEvent>,
}

impl Task {
    /// Track pump trade event
    pub async fn track_trade(&self, event: events::TradeEvent, con: &mut Connection) -> Result<()> {
        let mint = event.mint.to_string();
        if !con.exists(TaskCache::DevSoldOut(&mint))? {
            self.soldout.lock().await.insert(mint.clone());
        }

        for percent in [30, 10] {
            if !con.exists(TaskCache::Top10Holder {
                mint: &mint,
                percent: 30,
            })? {
                self.holders.lock().await.insert((mint.clone(), percent));
            }
        }

        Ok(())
    }
}
