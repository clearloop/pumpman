use crate::context::Context;
use teloxide::Bot;

/// Start the pumpman bot
pub async fn start(_bot: &Bot, _context: Context, _redis: String) -> anyhow::Result<()> {
    Ok(())
}
