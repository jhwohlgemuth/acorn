//! Audited JavaScript execution for the browser document.
//!
//! Every page-origin `document::eval` call belongs here. Other Rust modules call
//! a named function with a declared fire, query, or channel result shape.
//! Short JavaScript bodies use `js!`, which serializes placeholders as JSON
//! values and rejects placeholders in strings, templates, comments, regexes,
//! and identifier fragments. Bodies over 240 UTF-8 bytes use `js_file!` and
//! live in `assets/scripts/`, taking one `__acornArgs` JSON value.
//!
//! The default-off `js-lint` feature parses both macro forms with OxC at
//! compile time. A host test also checks the catalog and `window.acorn*` names
//! against their defining assets. Parsing and name checks do not type-check
//! browser APIs or prove that they are available at runtime.
// This module is the sole owner of browser evaluation; all callers use named scripts.
#![allow(clippy::disallowed_methods)]
use crate::dashboard::TileLayout;
use crate::workspace::BucketHistory;
use acorn_core::big_o::AsStr;
use acorn_macros::{js, js_file};
use acorn_web::geospatial::LocationIndex;
use acorn_web::session::SuppliedFile;
use core::future::Future;
use dioxus::prelude::{document, spawn};
use serde::de::DeserializeOwned;

/// Diagnostic name for a browser script event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EventName(String);
impl<const N: usize> From<[&str; N]> for EventName {
    fn from(parts: [&str; N]) -> Self {
        Self(parts.join("_"))
    }
}
impl AsStr for EventName {
    fn as_str(&self) -> &str {
        &self.0
    }
}
impl EventName {
    /// Run the named browser script and log failures against this event.
    pub(crate) fn fire(self, source: Result<String, serde_json::Error>) {
        fire(self, source);
    }
}
fn bound_channel(name: &'static str, source: Result<String, serde_json::Error>) -> document::Eval {
    match source {
        | Ok(source) => document::eval(&source),
        | Err(error) => {
            tracing::warn!(script = name, %error, "Browser channel serialization failed");
            document::eval("throw new Error('Browser channel serialization failed')")
        }
    }
}
pub(crate) fn cancel_remote_file_load() {
    EventName::from(["cancel", "remote", "file", "load"]).fire(js!("window.acornRemote?.file.cancel()"));
}
pub(crate) fn cancel_remote_load() {
    EventName::from(["cancel", "remote", "load"]).fire(js!("window.acornRemote?.cancel()"));
}
pub(crate) fn choose_theme(mode: &str) {
    EventName::from(["choose", "theme"]).fire(js!("window.acornChrome?.theme.choose({mode})"));
}
pub(crate) async fn clear_app_data() -> Result<(bool, bool, bool, bool), String> {
    query("clear_app_data", js_file!("clear-app-data.js", ())).await
}
pub(crate) async fn clear_appearance() -> Result<bool, String> {
    query("clear_appearance", js_file!("clear-appearance.js", ())).await
}
pub(crate) fn clear_code_link() {
    EventName::from(["clear", "code", "link"]).fire(js!("window.acornUrl.setParameters('code', null)"));
}
pub(crate) async fn clear_credential() -> Result<bool, String> {
    query("clear_credential", js!("return window.acornRemote?.clearCredential() ?? false")).await
}
pub(crate) fn clear_remote_history() {
    EventName::from(["clear", "remote", "history"]).fire(js!("window.acornRemote?.history.clear()"));
}
pub(crate) async fn clear_remote_history_counts() -> Result<(usize, usize), String> {
    query("clear_remote_history_counts", js!("window.acornRemote?.history.clear(); const history = window.acornRemote?.history.read(); return [history?.repositories.length ?? 0, history?.refs.length ?? 0];")).await
}
pub(crate) fn clear_template_link() {
    EventName::from(["clear", "template", "link"]).fire(js!("window.acornUrl.setParameters('template', null)"));
}
pub(crate) async fn clear_workspace_credential() -> Result<bool, String> {
    query("clear_workspace_credential", js!("const removed = window.acornRemote?.clearCredential() ?? false; const field = document.getElementById('repository-token'); if (field) field.value = ''; return removed;")).await
}
pub(crate) fn close_details(restore_focus: bool) {
    EventName::from(["close", "details"]).fire(js!("window.acornDetails?.close({restore_focus})"));
}
pub(crate) fn close_details_default() {
    EventName::from(["close", "details", "default"]).fire(js!("window.acornDetails?.close()"));
}
pub(crate) fn copy_text(text: &str) {
    EventName::from(["copy", "text"]).fire(js!("window.acornCopy({text})"));
}
pub(crate) async fn credential_status() -> Result<String, String> {
    query("credential_status", js!("if (!window.acornRemote) await new Promise(resolve => window.addEventListener('acorn-remote-ready', resolve, {once:true})); return window.acornRemote.credentialStatus();")).await
}
pub(crate) fn details_channel() -> document::Eval {
    bound_channel("details_channel", js!("if (!window.acornDetails) await new Promise(resolve => window.addEventListener('acorn-details-ready', resolve, {once:true})); window.acornDetails.connect(() => dioxus.send(null)); await dioxus.recv();"))
}
pub(crate) fn disconnect_map() {
    EventName::from(["disconnect", "map"]).fire(js!("window.acornMap?.disconnect()"));
}
pub(crate) fn disconnect_workspace_picker() {
    EventName::from(["disconnect", "workspace", "picker"]).fire(js!("window.acornWorkspace?.disconnect()"));
}
pub(crate) fn download_project(files: &[SuppliedFile]) {
    EventName::from(["download", "project"]).fire(js_file!("download-project.js", files));
}
pub(crate) fn download_result(content: &str, filename: &str, mime: &str) {
    let args = (content, filename, mime);
    EventName::from(["download", "result"]).fire(js!(
        "const [content, filename, mime] = {args}; window.acornDownload(content, filename, mime)"
    ));
}
pub(crate) fn download_skill_definition(content: &str) {
    EventName::from(["download", "skill", "definition"]).fire(js!("window.acornDownload({content}, 'SKILL.md', 'text/plain;charset=utf-8')"));
}
pub(crate) fn download_skill_folder(base: &str, version: &str) {
    let args = (base, version);
    EventName::from(["download", "skill", "folder"]).fire(js!("const [base, version] = {args}; window.acornSkill.download(base, version)"));
}
pub(crate) fn download_workflow_prompt(prompt: &str, filename: &str) {
    let args = (prompt, filename, "text/plain;charset=utf-8");
    EventName::from(["download", "workflow", "prompt"]).fire(
        js!("const [prompt, filename, mime] = {args}; window.acornDownload(prompt, filename, mime); document.querySelectorAll('.workflow-menu[open]').forEach(menu => menu.removeAttribute('open'))"),
    );
}
pub(crate) fn download_workspace(content: &str) {
    EventName::from(["download", "workspace"]).fire(js!("window.acornDownload({content}, 'acorn-workspace.json')"));
}
pub(crate) fn expand_editor(id: &'static str) {
    EventName::from(["expand", "editor"]).fire(js!("window.acornEditors?.expand({id})"));
}
pub(crate) fn fire(name: EventName, source: Result<String, serde_json::Error>) {
    match source {
        | Ok(source) => {
            let observed = format!("{source};\nreturn null;");
            let eval = document::eval(&observed);
            spawn(async move {
                if let Err(error) = eval.join::<serde_json::Value>().await {
                    tracing::warn!(script = name.as_str(), %error, "Browser script failed");
                }
            });
        }
        | Err(error) => tracing::warn!(script = name.as_str(), %error, "Browser script serialization failed"),
    }
}
pub(crate) fn focus_content() {
    EventName::from(["focus", "content"]).fire(js!(
        "(document.getElementById('content-editor') || document.getElementById('content'))?.focus()"
    ));
}
pub(crate) fn focus_content_editor() {
    EventName::from(["focus", "content", "editor"]).fire(js!("requestAnimationFrame(() => document.getElementById('content-editor')?.focus())"));
}
pub(crate) fn focus_content_next_frame() {
    EventName::from(["focus", "content", "next", "frame"]).fire(js!(
        "requestAnimationFrame(() => (document.getElementById('content-editor') || document.getElementById('content'))?.focus())"
    ));
}
pub(crate) fn focus_dashboard_settings() {
    EventName::from(["focus", "dashboard", "settings"]).fire(js!("document.querySelector('.dashboard-settings > summary')?.focus()"));
}
pub(crate) fn focus_home_source_trigger() {
    EventName::from(["focus", "home", "source", "trigger"]).fire(js!("document.getElementById('home-source-trigger')?.focus()"));
}
pub(crate) fn focus_main_heading() {
    EventName::from(["focus", "main", "heading"]).fire(js!("requestAnimationFrame(() => document.querySelector('#content-main h1')?.focus())"));
}
pub(crate) fn focus_nav_link() {
    EventName::from(["focus", "nav", "link"]).fire(js!(
        "requestAnimationFrame(() => document.querySelector('#workspace-navigation a')?.focus())"
    ));
}
pub(crate) fn focus_nav_toggle() {
    EventName::from(["focus", "nav", "toggle"]).fire(js!("document.getElementById('nav-toggle')?.focus()"));
}
pub(crate) fn focus_template_create() {
    EventName::from(["focus", "template", "create"]).fire(js!(
        "requestAnimationFrame(() => document.querySelector('.template-prompt .template-create')?.focus())"
    ));
}
pub(crate) fn focus_template_trigger(id: &str) {
    EventName::from(["focus", "template", "trigger"]).fire(
        js!("requestAnimationFrame(() => { const card = [...document.querySelectorAll('[data-template-id]')].find(node => node.dataset.templateId === {id}); (card || document.querySelector('[data-template-trigger]'))?.focus(); })"),
    );
}
pub(crate) fn focus_url_file_input() {
    EventName::from(["focus", "url", "file", "input"]).fire(js!(
        "requestAnimationFrame(() => { const input = document.querySelector('.file-picker .url-file-loader input'); if (input) input.focus(); })"
    ));
}
pub(crate) fn focus_url_loader() {
    EventName::from(["focus", "url", "loader"]).fire(js!(
        "requestAnimationFrame(() => document.querySelector('.url-file-loader input')?.focus())"
    ));
}
pub(crate) fn fold_editor(id: &'static str) {
    EventName::from(["fold", "editor"]).fire(js!("window.acornEditors?.fold({id})"));
}
pub(crate) async fn highlight(text: &str, language: &str) -> Result<String, String> {
    let args = (text, language);
    query("highlight", js!("if (!window.acornSyntax) await new Promise(resolve => window.addEventListener('acorn-syntax-ready', resolve, {once:true})); const [text, language] = {args}; return window.acornSyntax.highlight(text, language);")).await
}
pub(crate) async fn load_dashboard_layout() -> Result<Option<TileLayout>, String> {
    query("load_dashboard_layout", js!("if (!window.acornDashboardLayout) await new Promise(resolve => window.addEventListener('acorn-dashboard-layout-ready', resolve, {once:true})); return window.acornDashboardLayout.load('portfolio')")).await
}
pub(crate) async fn load_remote_bucket(url: &str, reference: &str) -> Result<serde_json::Value, String> {
    let args = (url, reference);
    query("load_remote_bucket", js_file!("load-remote-bucket.js", args)).await
}
pub(crate) async fn load_ror_index(url: &str) -> Result<LocationIndex, String> {
    query("load_ror_index", js_file!("load-ror-index.js", url)).await
}
pub(crate) async fn load_skill(base: &str, version: &str) -> Result<serde_json::Value, String> {
    let args = (base, version);
    query("load_skill", js_file!("load-skill.js", args)).await
}
pub(crate) async fn load_url_file(url: &str, limit: usize, workspace_file: bool) -> Result<serde_json::Value, String> {
    let args = (url, limit, workspace_file);
    query("load_url_file", js_file!("load-url-file.js", args)).await
}
pub(crate) fn map_channel(module: &str, css: &str) -> document::Eval {
    let args = (module, css);
    bound_channel("map_channel", js_file!("map-channel.js", args))
}

pub(crate) fn navigate_editor_issue(id: &'static str, direction: i8) {
    EventName::from(["navigate", "editor", "issue"]).fire(js!("window.acornEditors?.navigateIssue({id}, {direction})"));
}
pub(crate) fn notify_dashboard(message: &str, kind: &'static str) {
    EventName::from(["notify", "dashboard"]).fire(js!("window.acornChrome?.notifications.notify({message}, {kind})"));
}
pub(crate) fn notify_sample_loaded() {
    EventName::from(["notify", "sample", "loaded"]).fire(js!(
        "window.acornChrome?.notifications.notify('Synthetic research bucket loaded.', 'success')"
    ));
}
pub(crate) fn notify_storage(key: &str) {
    EventName::from(["notify", "storage"]).fire( js!("if (!window.acornWindowSync) await new Promise(resolve => window.addEventListener('acorn-window-sync-ready', resolve, {once:true})); window.acornWindowSync.notifyStorage({key});"));
}
pub(crate) fn notify_workspace_deleted() {
    EventName::from(["notify", "workspace", "deleted"]).fire(js!(
        "window.acornChrome?.theme.reset(); window.acornChrome?.notifications.notify('All app data deleted.', 'success')"
    ));
}
pub(crate) fn notify_workspace_save_failure(message: &str) {
    EventName::from(["notify", "workspace", "save", "failure"]).fire(js!("window.acornChrome?.notifications.notify({message}, 'warning')"));
}
pub(crate) fn open_details() {
    EventName::from(["open", "details"]).fire( js!("if (!window.acornDetails) await new Promise(resolve => window.addEventListener('acorn-details-ready', resolve, {once:true})); requestAnimationFrame(() => window.acornDetails.open())"));
}
pub(crate) fn open_editor_find(id: &'static str) {
    EventName::from(["open", "editor", "find"]).fire(js!("window.acornEditors?.openFind({id})"));
}
pub(crate) fn open_editor_in_new_tab(id: &'static str, state: Option<String>) {
    EventName::from(["open", "editor", "in", "new", "tab"]).fire(js!("window.acornEditors?.openInNewTab({id}, {state})"));
}
pub(crate) fn sync_popout_input(id: &'static str, value: &str) {
    EventName::from(["sync", "popout", "input"]).fire(js!("window.acornEditors?.syncPopoutInput({id}, {value})"));
}
pub(crate) fn open_file_picker(slug: &'static str) {
    EventName::from(["open", "file", "picker"]).fire(js!(
        "const picker = document.getElementById('source-file-' + {slug}); if (picker) { picker.value = ''; picker.click(); }"
    ));
}
pub(crate) fn operation_channel() -> document::Eval {
    bound_channel("operation_channel", js!("const message = await dioxus.recv(); return await (window.acornOperations?.run(message) ?? { unavailable: 'The operation client is unavailable.' });"))
}
pub(crate) fn pick_workspace_file() {
    EventName::from(["pick", "workspace", "file"]).fire(js!("window.acornWorkspace.pick('file')"));
}
pub(crate) fn pick_workspace_folder() {
    EventName::from(["pick", "workspace", "folder"]).fire(js!("window.acornWorkspace.pick('folder')"));
}
pub(crate) fn publish_json(topic: &str, payload: &str) {
    let source = serde_json::from_str::<serde_json::Value>(payload).and_then(|payload| js!("if (!window.acornWindowSync) await new Promise(resolve => window.addEventListener('acorn-window-sync-ready', resolve, {once:true})); window.acornWindowSync.publish({topic}, {payload});"));
    EventName::from(["publish", "json"]).fire(source);
}
pub(crate) fn query<T: DeserializeOwned>(name: &'static str, source: Result<String, serde_json::Error>) -> impl Future<Output = Result<T, String>> {
    let eval = source.map(|source| document::eval(&source)).map_err(|error| format!("{name}: {error}"));
    async move {
        match eval {
            | Ok(eval) => eval.join::<T>().await.map_err(|error| format!("{name}: {error}")),
            | Err(error) => Err(error),
        }
    }
}
pub(crate) async fn read_remote_history() -> Result<BucketHistory, String> {
    query("read_remote_history", js!("if (!window.acornRemote) await new Promise(resolve => window.addEventListener('acorn-remote-ready', resolve, {once:true})); return window.acornRemote.history.read();")).await
}
pub(crate) async fn read_remote_history_counts() -> Result<(usize, usize), String> {
    query("read_remote_history_counts", js!("if (!window.acornRemote) await new Promise(resolve => window.addEventListener('acorn-remote-ready', resolve, {once:true})); const history = window.acornRemote.history.read(); return [history.repositories.length, history.refs.length];")).await
}
pub(crate) async fn read_storage_status() -> Result<serde_json::Value, String> {
    query("read_storage_status", js!("if (!window.acornWorkspaceStorage) await new Promise(resolve => window.addEventListener('acorn-workspace-storage-ready', resolve, {once:true})); return await window.acornWorkspaceStorage.status()")).await
}
pub(crate) async fn refresh_storage_status() -> Result<serde_json::Value, String> {
    query("refresh_storage_status", js!("return await window.acornWorkspaceStorage.status()")).await
}
pub(crate) async fn reload_dashboard_layout() -> Result<Option<TileLayout>, String> {
    query("reload_dashboard_layout", js!("return window.acornDashboardLayout.load('portfolio')")).await
}
pub(crate) fn remember_details_focus() {
    EventName::from(["remember", "details", "focus"]).fire(js!("window.acornDetails?.rememberFocus()"));
}
pub(crate) fn request_editor_state(topic: &str, after: u64) {
    EventName::from(["request", "editor", "state"]).fire(js!(
        "window.opener?.postMessage({type: 'acorn-editor-state-refresh', topic: {topic}, after: {after}}, location.origin)"
    ));
}
pub(crate) async fn reset_dashboard_layout() -> Result<bool, String> {
    query(
        "reset_dashboard_layout",
        js!("return window.acornDashboardLayout?.reset('portfolio') ?? false"),
    )
    .await
}
pub(crate) async fn restore_saved_workspace() -> Result<serde_json::Value, String> {
    query("restore_saved_workspace", js!("return await window.acornWorkspaceStorage.restore()")).await
}
pub(crate) async fn restore_workspace_snapshot() -> Result<serde_json::Value, String> {
    query("restore_workspace_snapshot", js_file!("restore-workspace-snapshot.js", ())).await
}
pub(crate) fn reveal_diagnostic(line: u64, character: u64, end_line: u64, end_character: u64) {
    EventName::from(["reveal", "diagnostic"]).fire(js_file!("reveal-diagnostic.js", (line, character, end_line, end_character)));
}
pub(crate) fn review_fragments_channel() -> document::Eval {
    bound_channel("review_fragments_channel", js_file!("review-fragments-channel.js", ()))
}
pub(crate) fn run_editor_tool() {
    EventName::from(["run", "editor", "tool"]).fire(js!("document.getElementById('tool-run')?.click()"));
}
pub(crate) async fn save_dashboard_layout(layout: &TileLayout) -> Result<bool, String> {
    query(
        "save_dashboard_layout",
        js!("return window.acornDashboardLayout?.save('portfolio', {layout}) ?? false"),
    )
    .await
}
pub(crate) fn save_workspace(value: &str) -> impl Future<Output = Result<serde_json::Value, String>> {
    let source = serde_json::from_str::<serde_json::Value>(value).and_then(|value| js!("const storage = window.acornWorkspaceStorage; const saved = await storage.save({value}); return {...await storage.status(), newFailure: !saved}"));
    query("save_workspace", source)
}
pub(crate) fn scroll_choice_into_view(label: &'static str) {
    EventName::from(["scroll", "choice", "into", "view"]).fire(js_file!("scroll-choice-into-view.js", label));
}
pub(crate) fn scroll_details_to_top() {
    EventName::from(["scroll", "details", "to", "top"]).fire( js!("requestAnimationFrame(() => { const panel = document.getElementById('details-drawer'); if (panel?.open) { panel.querySelector('.details-body')?.scrollTo(0, 0); panel.querySelector('.details-close')?.focus({preventScroll:true}); } })"));
}
pub(crate) fn set_editor_font_size(id: &'static str, size: &'static str) {
    EventName::from(["set", "editor", "font", "size"]).fire(js!("window.acornEditors?.setFontSize({id}, {size})"));
}
pub(crate) fn set_review_fragment(fragment: &'static str) {
    EventName::from(["set", "review", "fragment"]).fire( js!("if (!window.acornUrl) await new Promise(resolve => window.addEventListener('acorn-url-ready', resolve, {once:true})); window.acornInitialReviewFragment = ''; window.acornUrl.setFragment({fragment})"));
}
pub(crate) fn set_sidebar_collapsed(collapsed: bool) {
    EventName::from(["set", "sidebar", "collapsed"]).fire(js!(
        "try { sessionStorage.setItem('acorn-sidebar-collapsed', String({collapsed})); } catch {}"
    ));
}
pub(crate) async fn set_workspace_credential() -> Result<serde_json::Value, String> {
    query("set_workspace_credential", js!("try { const field = document.getElementById('repository-token'); const status = window.acornRemote.setCredential(field.value); field.value = ''; return {status: status}; } catch(error) { return {error: error.message}; }")).await
}
pub(crate) fn share_template_link(id: &str) {
    EventName::from(["share", "template", "link"]).fire(js!(
        "window.acornUrl.setParameters('template', {id}); window.acornCopy(window.acornUrl.getHref())"
    ));
}
pub(crate) fn show_tool_result(slug: &'static str, celebrate: bool) {
    EventName::from(["show", "tool", "result"]).fire(js_file!("show-tool-result.js", (slug, celebrate)));
}
pub(crate) fn start_tour() {
    EventName::from(["start", "tour"]).fire(js!("window.acornChrome?.tour.start()"));
}
pub(crate) fn subscribe_storage_channel(key: &str) -> document::Eval {
    bound_channel("subscribe_storage_channel", js_file!("subscribe-storage-channel.js", key))
}
pub(crate) fn subscribe_window_sync_channel(topic: &str) -> document::Eval {
    bound_channel("subscribe_window_sync_channel", js_file!("subscribe-window-sync-channel.js", topic))
}
pub(crate) fn sync_editor(
    id: &'static str,
    replacement: Option<String>,
    language: String,
    label: String,
    diagnostics: Vec<serde_json::Value>,
    source: String,
) {
    let args = (id, replacement, language, label, diagnostics, source);
    EventName::from(["sync", "editor"]).fire(js_file!("sync-editor.js", args));
}
pub(crate) fn sync_page_navigation() {
    EventName::from(["sync", "page", "navigation"]).fire(js_file!("sync-page-navigation.js", ()));
}
pub(crate) fn toggle_editor_fullscreen(id: &'static str) {
    EventName::from(["toggle", "editor", "fullscreen"]).fire(js!("window.acornEditors?.toggleFullscreen({id})"));
}
pub(crate) fn toggle_editor_wrap(id: &'static str) {
    EventName::from(["toggle", "editor", "wrap"]).fire(js!("window.acornEditors?.toggleWrap({id})"));
}
pub(crate) fn toggle_theme() {
    EventName::from(["toggle", "theme"]).fire(js!("window.acornChrome?.theme.toggle()"));
}
pub(crate) fn update_map(payload: &serde_json::Value) {
    EventName::from(["update", "map"]).fire(js!("window.acornMap?.update({payload})"));
}
pub(crate) fn url_entry_channel() -> document::Eval {
    bound_channel("url_entry_channel", js_file!("url-entry-channel.js", ()))
}
pub(crate) fn warm_operations() {
    EventName::from(["warm", "operations"]).fire(js!("window.acornOperations?.warm()"));
}
pub(crate) fn workspace_picker_channel() -> document::Eval {
    bound_channel("workspace_picker_channel", js!("if (!window.acornWorkspace) await new Promise(resolve => window.addEventListener('acorn-effects-ready', resolve, {once:true})); window.acornWorkspace.connect(batch => dioxus.send(batch)); await dioxus.recv();"))
}
pub(crate) async fn yield_main_thread() -> Result<serde_json::Value, String> {
    query(
        "yield_main_thread",
        js!("await new Promise(resolve => requestAnimationFrame(() => setTimeout(resolve))); return null"),
    )
    .await
}

#[cfg(all(test, feature = "js-lint", not(target_arch = "wasm32")))]
mod lint;
