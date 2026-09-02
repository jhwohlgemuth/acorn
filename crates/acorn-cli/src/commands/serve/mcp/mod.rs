use color_eyre::eyre::{Report, Result};

pub async fn run(offline: bool) -> Result<(), Report> {
    acorn::io::mcp::run(offline).await
}
