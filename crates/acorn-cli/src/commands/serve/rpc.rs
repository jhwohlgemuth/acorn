use acorn::io::api::{self, json_rpc};
use acorn::util::constants::env::RPC_TOKEN;
use color_eyre::eyre::{eyre, Report, Result};
use core::net::SocketAddr;
use std::env::var;

pub async fn run(address: SocketAddr, allow_mutation: bool, offline: bool) -> Result<(), Report> {
    let token = var(RPC_TOKEN)
        .ok()
        .filter(|token| !token.trim().is_empty())
        .map(api::Secret::from)
        .ok_or_else(|| eyre!("{RPC_TOKEN} is required to serve JSON-RPC"));
    match token {
        | Ok(token) => json_rpc::http::serve(address, token, allow_mutation, offline).await,
        | Err(why) => Err(why),
    }
}
