

use std::io::{self, BufRead, BufWriter, Write};
use std::thread;
use std::time::Duration;
const CONFIG_OPTIONS: &str = r#"[{"id":"model","name":"Model","category":"model","type":"select","currentValue":"fake-model","options":[{"value":"fake-model","name":"Fake Model"}]}]"#;
fn main() {
    let argument = std::env::args().nth(1).unwrap_or_default();
    match argument.as_str() {
        "--version" => println!("fake-acp 1.2.3"),
        "acp" => serve(),
        "fail-version" => {
            eprintln!("version probe failed");
            std::process::exit(7);
        }
        "slow-version" => {
            thread::sleep(Duration::from_secs(3));
            println!("fake-acp 1.2.3");
        }
        _ => std::process::exit(2),
    }
}
fn request_id(line: &str) -> String {
    match line.find("\"id\":") {
        Some(position) => {
            let value = line[position + 5..].trim_start();
            match value.starts_with('"') {
                true => value[1..]
                    .find('"')
                    .map(|end| format!("\"{}\"", &value[1..end + 1]))
                    .unwrap_or_else(|| "null".to_string()),
                false => value
                    .chars()
                    .take_while(|character| character.is_ascii_digit() || *character == '-')
                    .collect(),
            }
        }
        None => "null".to_string(),
    }
}
fn send(writer: &mut BufWriter<io::StdoutLock<'_>>, message: &str) {
    writeln!(writer, "{message}").expect("write fake ACP response");
    writer.flush().expect("flush fake ACP response");
}
fn send_chunk(writer: &mut BufWriter<io::StdoutLock<'_>>, text: &str) {
    let escaped = text.replace('\\', "\\\\").replace('"', "\\\"");
    send(
        writer,
        &format!(
            r#"{{"jsonrpc":"2.0","method":"session/update","params":{{"sessionId":"fake-session","update":{{"sessionUpdate":"agent_message_chunk","content":{{"type":"text","text":"{escaped}"}}}}}}}}"#,
        ),
    );
}
fn serve() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());
    let mut permission_prompt = None;
    for line in stdin.lock().lines().map_while(Result::ok) {
        let id = request_id(&line);
        match () {
            _ if line.contains("\"method\":\"initialize\"") => send(
                &mut writer,
                &format!(
                    r#"{{"jsonrpc":"2.0","id":{id},"result":{{"protocolVersion":1,"agentCapabilities":{{"promptCapabilities":{{"image":true}},"mcpCapabilities":{{"http":true}}}},"authMethods":[],"agentInfo":{{"name":"fake-acp","version":"1.0.0"}}}}}}"#,
                ),
            ),
            _ if line.contains("\"method\":\"session/new\"") => send(
                &mut writer,
                &format!(
                    r#"{{"jsonrpc":"2.0","id":{id},"result":{{"sessionId":"fake-session","configOptions":{CONFIG_OPTIONS}}}}}"#,
                ),
            ),
            _ if line.contains("\"method\":\"session/set_config_option\"") => send(
                &mut writer,
                &format!(r#"{{"jsonrpc":"2.0","id":{id},"result":{{"configOptions":{CONFIG_OPTIONS}}}}}"#),
            ),
            _ if line.contains("\"method\":\"session/prompt\"") && line.contains("\"text\":\"hang\"") => {
                thread::sleep(Duration::from_secs(3));
                send(&mut writer, &format!(r#"{{"jsonrpc":"2.0","id":{id},"result":{{"stopReason":"end_turn"}}}}"#));
            }
            _ if line.contains("\"method\":\"session/prompt\"") && line.contains("early-exit") => {
                eprintln!("fake agent exited during prompt");
                std::process::exit(9);
            }
            _ if line.contains("\"method\":\"session/prompt\"") && line.contains("\"text\":\"permission\"") => {
                permission_prompt = Some(id);
                send(
                    &mut writer,
                    r#"{"jsonrpc":"2.0","id":900,"method":"session/request_permission","params":{"sessionId":"fake-session","toolCall":{"toolCallId":"call-1"},"options":[{"optionId":"reject","name":"Reject","kind":"reject_once"}]}}"#,
                );
            }
            _ if line.contains("\"method\":\"session/prompt\"")
                && line.contains("# Input Data (ZON)")
                && line.contains(r#"```zon\ntext:ZON-PASSTHROUGH\n```"#) =>
            {
                send_chunk(&mut writer, "zon prompt received");
                send(&mut writer, &format!(r#"{{"jsonrpc":"2.0","id":{id},"result":{{"stopReason":"end_turn"}}}}"#));
            }
            _ if line.contains("\"method\":\"session/prompt\"") && line.contains("check-0001") => {
                send_chunk(
                    &mut writer,
                    r#"{"fixes":[{"actions":[{"instructions":"Replace the title with a valid value","kind":"manual"}],"automation":"manual","confidence":0.9,"effects":{"elevation":false,"filesystem_write":false,"network":false,"package_manager":false,"process":false,"restart":false,"reversible":true},"finding_id":"check-0001","id":"fix-0001","rationale":"The title requires human review.","risk":"low","source_fingerprint":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","status":"proposed","summary":"Review the title","target":"activity.json"}],"version":"1"}"#,
                );
                send(&mut writer, &format!(r#"{{"jsonrpc":"2.0","id":{id},"result":{{"stopReason":"end_turn"}}}}"#));
            }
            _ if line.contains("\"method\":\"session/prompt\"") && line.contains("structured") => {
                send_chunk(&mut writer, r#"{"answer":42}"#);
                send(&mut writer, &format!(r#"{{"jsonrpc":"2.0","id":{id},"result":{{"stopReason":"end_turn"}}}}"#));
            }
            _ if line.contains("\"method\":\"session/prompt\"") => {
                send_chunk(&mut writer, "fake ");
                send_chunk(&mut writer, "response");
                send(&mut writer, &format!(r#"{{"jsonrpc":"2.0","id":{id},"result":{{"stopReason":"end_turn"}}}}"#));
            }
            _ if line.contains("\"id\":900") && line.contains("\"outcome\":\"selected\"") => match permission_prompt.take() {
                Some(prompt_id) => send(
                    &mut writer,
                    &format!(r#"{{"jsonrpc":"2.0","id":{prompt_id},"result":{{"stopReason":"cancelled"}}}}"#),
                ),
                None => {}
            },
            _ => {}
        }
    }
}
