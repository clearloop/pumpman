// Day in seconds
pub const DAY: u64 = 86400;

pub mod base64 {
    use anyhow::Result;
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    /// base64 decode
    pub fn decode(encoded: &str) -> Result<Vec<u8>> {
        STANDARD.decode(encoded).map_err(Into::into)
    }
}
