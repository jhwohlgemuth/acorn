//! Run a user-defined Rhai prompt through OpenCode or Codex over ACP
use acorn::agent::{InferencePermissionPolicy, InferenceRequest, PromptContext, PromptScriptHydrator};
use acorn::io::acp::{AgentProfile, Client, Codex, OpenCode};
use acorn_schema::agent::PromptTemplateConfiguration;
use color_eyre::eyre::{eyre, Result};
use serde_json::json;
use std::{env, fs};

const SCRIPT: &str = r#"// @acorn-exec-v0.2.0
fn evaluate(context) {
    "Give one concise title for this research activity: " + context.data.description
}
"#;
fn main() -> Result<()> {
    let mut arguments = env::args().skip(1);
    let agent = arguments.next().unwrap_or_else(|| "opencode".to_string());
    let path = arguments.next();
    let context = PromptContext {
        config: PromptTemplateConfiguration::default(),
        data: json!({"description": "A field survey of water retention and amphibian calls in three wetland plots"}),
    };
    let label = path.as_deref().unwrap_or("title.rhai");
    env::current_dir()
        .map_err(Into::into)
        .and_then(|working_directory| match agent.as_str() {
            | "codex" => Ok(Codex::process_config(None, working_directory)),
            | "opencode" => Ok(OpenCode::process_config(None, working_directory)),
            | _ => Err(eyre!("Expected agent 'opencode' or 'codex'")),
        })
        .and_then(|config| {
            path.as_ref()
                .map(fs::read_to_string)
                .transpose()
                .map(|source| source.unwrap_or_else(|| SCRIPT.to_string()))
                .map_err(Into::into)
                .and_then(|source| PromptScriptHydrator::hydrate(label, &source, &context))
                .and_then(|prompt| {
                    let request = InferenceRequest {
                        agent: Some(config.agent_id().to_string()),
                        model: None,
                        permission_policy: InferencePermissionPolicy::DenyMutation,
                        prompt,
                        response_schema: None,
                        timeout: 30_000.into(),
                    };
                    tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .map_err(Into::into)
                        .and_then(|runtime| runtime.block_on(Client::new(config).infer_checked(&request)).map_err(Into::into))
                })
        })
        .map(|result| println!("{}", result.text))
}
