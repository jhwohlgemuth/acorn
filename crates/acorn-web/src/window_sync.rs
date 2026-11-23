//! Browser messaging and cross-window state synchronization.
use acorn_web::operations::{Request, Tool};
use acorn_web::session::SuppliedFile;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub(crate) fn notify_storage(key: &str) {
    let key = serde_json::to_string(key).unwrap_or_default();
    document::eval(&format!("if (!window.acornWindowSync) await new Promise(resolve => window.addEventListener('acorn-window-sync-ready', resolve, {{once:true}})); window.acornWindowSync.notifyStorage({key});"));
}
pub(crate) fn publish_json(topic: &str, payload: &str) {
    let topic = serde_json::to_string(topic).unwrap_or_default();
    document::eval(&format!("if (!window.acornWindowSync) await new Promise(resolve => window.addEventListener('acorn-window-sync-ready', resolve, {{once:true}})); window.acornWindowSync.publish({topic}, {payload});"));
}
pub(crate) fn subscribe(topic: &str) -> document::Eval {
    let topic = serde_json::to_string(topic).unwrap_or_default();
    document::eval(&format!("if (!window.acornWindowSync) await new Promise(resolve => window.addEventListener('acorn-window-sync-ready', resolve, {{once:true}})); const unsubscribe = window.acornWindowSync.subscribe({topic}, message => dioxus.send(message)); await dioxus.recv(); unsubscribe();"))
}
pub(crate) fn subscribe_storage(key: &str) -> document::Eval {
    let key = serde_json::to_string(key).unwrap_or_default();
    document::eval(&format!("if (!window.acornWindowSync) await new Promise(resolve => window.addEventListener('acorn-window-sync-ready', resolve, {{once:true}})); const unsubscribe = window.acornWindowSync.subscribeStorage({key}, () => dioxus.send(true)); await dioxus.recv(); unsubscribe();"))
}

pub(crate) fn fragments() -> document::Eval {
    document::eval("if (!window.acornUrl) await new Promise(resolve => window.addEventListener('acorn-url-ready', resolve, {once:true})); if (!window.acornWindowSync) await new Promise(resolve => window.addEventListener('acorn-window-sync-ready', resolve, {once:true})); const initial = window.acornInitialReviewFragment; if (initial && !window.acornUrl.getFragment()) window.acornUrl.setFragment(initial, true); window.acornInitialReviewFragment = ''; const receive = window.acornWindowSync.subscribe('acorn.review.mode.v1', fragment => { if (typeof fragment === 'string' && fragment !== window.acornUrl.getFragment()) window.acornUrl.setFragment(fragment, true); }); const unsubscribe = window.acornUrl.subscribe(fragment => { window.acornWindowSync.publish('acorn.review.mode.v1', fragment); dioxus.send(fragment); }); await dioxus.recv(); unsubscribe(); receive();")
}
#[derive(Clone, Copy)]
pub(crate) struct ToolSignals {
    pub(crate) drafts: Signal<BTreeMap<String, (String, String)>>,
    pub(crate) editor_source: Signal<Option<SuppliedFile>>,
    pub(crate) last_request: Signal<Request>,
    pub(crate) request: Signal<Request>,
    pub(crate) result: Signal<Option<Result<Value, String>>>,
    pub(crate) running: Signal<bool>,
    pub(crate) seen_session: Signal<String>,
}
#[derive(Clone, Deserialize, PartialEq, Serialize)]
struct ToolSnapshot {
    drafts: BTreeMap<String, (String, String)>,
    editor_source: Option<SuppliedFile>,
    request: Request,
    result: Option<Result<Value, String>>,
}
#[derive(Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ToolMessage {
    Request,
    State { revision: u64, snapshot: Box<ToolSnapshot> },
}
impl ToolSnapshot {
    fn is_pristine(&self, session: &str) -> bool {
        self.drafts.is_empty()
            && self.result.is_none()
            && self.editor_source.as_ref().is_none_or(|source| source.content == session)
            && self.request
                == Request {
                    input: session.to_owned(),
                    ..Request::default()
                }
    }
}
impl ToolSignals {
    pub(crate) fn new(
        request: Signal<Request>,
        last_request: Signal<Request>,
        seen_session: Signal<String>,
        drafts: Signal<BTreeMap<String, (String, String)>>,
        editor_source: Signal<Option<SuppliedFile>>,
    ) -> Self {
        Self {
            drafts,
            editor_source,
            last_request,
            request,
            result: Signal::new(None),
            running: Signal::new(false),
            seen_session,
        }
    }
    pub(crate) fn isolated() -> Self {
        Self::new(
            Signal::new(Request::default()),
            Signal::new(Request::default()),
            Signal::new(String::new()),
            Signal::new(BTreeMap::new()),
            Signal::new(None),
        )
    }
    fn snapshot(self) -> ToolSnapshot {
        ToolSnapshot {
            drafts: self.drafts.peek().clone(),
            editor_source: self.editor_source.peek().clone(),
            request: self.request.peek().clone(),
            result: self.result.peek().clone(),
        }
    }
}
fn publish(tool: Tool, message: &ToolMessage) {
    if let Ok(payload) = serde_json::to_string(message) {
        publish_json(&format!("acorn.tool.{}", tool.slug()), &payload);
    }
}
pub(crate) fn use_tool_sync(tool: Tool, mut signals: ToolSignals, session: Signal<String>) {
    let mut ready_for = use_signal(|| None::<Tool>);
    let mut received = use_signal(|| None::<(Tool, ToolSnapshot)>);
    let mut baseline = use_signal(|| None::<(Tool, ToolSnapshot)>);
    let mut pending = use_signal(|| None::<(Tool, u64, ToolSnapshot)>);
    let mut revision = use_signal(|| 0_u64);
    use_future(use_reactive((&tool,), move |(tool,)| async move {
        ready_for.set(None);
        received.set(None);
        baseline.set(None);
        pending.set(None);
        revision.set(0);
        let topic = format!("acorn.tool.{}", tool.slug());
        let mut channel = subscribe(&topic);
        while let Ok(value) = channel.recv::<Value>().await {
            let Ok(message) = serde_json::from_value::<ToolMessage>(value) else {
                continue;
            };
            match message {
                | ToolMessage::Request if *ready_for.peek() == Some(tool) => {
                    let snapshot = signals.snapshot();
                    if !snapshot.is_pristine(&session.peek()) || *revision.peek() > 0 {
                        publish(
                            tool,
                            &ToolMessage::State {
                                revision: *revision.peek(),
                                snapshot: Box::new(snapshot),
                            },
                        );
                    }
                }
                | ToolMessage::State {
                    revision: incoming,
                    snapshot,
                } if incoming > *revision.peek() || (received.peek().is_none() && *revision.peek() == 0) => {
                    if pending.peek().as_ref().is_none_or(|(_, queued, _)| incoming >= *queued) {
                        pending.set(Some((tool, incoming, *snapshot)));
                    }
                }
                | _ => {}
            }
        }
    }));
    use_effect(move || {
        let current_session = session();
        if (signals.seen_session)() != current_session {
            return;
        }
        if let Some((tracked, incoming, snapshot)) = pending() {
            if tracked == tool && (incoming > *revision.peek() || (received.peek().is_none() && *revision.peek() == 0)) {
                revision.set(incoming);
                received.set(Some((tool, snapshot.clone())));
                baseline.set(Some((tool, snapshot.clone())));
                signals.last_request.set(snapshot.request.clone());
                signals.drafts.set(snapshot.drafts);
                signals.editor_source.set(snapshot.editor_source);
                signals.request.set(snapshot.request);
                signals.result.set(snapshot.result);
                ready_for.set(Some(tool));
            }
            pending.set(None);
            return;
        }
        let snapshot = ToolSnapshot {
            drafts: (signals.drafts)(),
            editor_source: (signals.editor_source)(),
            request: (signals.request)(),
            result: (signals.result)(),
        };
        let observed_baseline = baseline();
        if observed_baseline.as_ref().is_none_or(|(tracked, _)| *tracked != tool) {
            baseline.set(Some((tool, snapshot)));
            ready_for.set(Some(tool));
            publish(tool, &ToolMessage::Request);
            return;
        }
        if observed_baseline != Some((tool, snapshot.clone())) {
            let millis = u64::try_from(js_sys::Date::now() as i64).unwrap_or_default();
            let next = millis.saturating_mul(1000).max(revision.peek().saturating_add(1));
            revision.set(next);
            baseline.set(Some((tool, snapshot.clone())));
            publish(
                tool,
                &ToolMessage::State {
                    revision: next,
                    snapshot: Box::new(snapshot),
                },
            );
        }
    });
}
