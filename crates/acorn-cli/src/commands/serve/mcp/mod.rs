use acorn::io::mcp;
use color_eyre::eyre::{Report, Result};

pub async fn run(offline: bool) -> Result<(), Report> {
    mcp::run(offline).await
}
