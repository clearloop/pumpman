use crate::{config::PumpmanGlobal, Context};
use std::sync::Arc;

/// Wrapped context
#[derive(Clone)]
pub struct PumpmanContext {
    /// command context
    pub context: Context,
    /// cutomized data in context
    pub global: Arc<PumpmanGlobal>,
}

impl PumpmanContext {
    /// Create new wrapped context
    pub fn new(context: Context, global: PumpmanGlobal) -> Self {
        Self {
            context,
            global: Arc::new(global),
        }
    }
}
