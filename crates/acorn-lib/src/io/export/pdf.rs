//! Shared Chromium PDF rendering session and executable policy.
use crate::io::ApiResult;
use crate::util::constants::app::{APPLICATION, CHROME_VERSION};
use chromiumoxide::{
    browser::{Browser, BrowserConfig},
    cdp::browser_protocol::page::PrintToPdfParams,
    fetcher::{BrowserFetcher, BrowserFetcherOptions, BrowserKind, Version},
    Page,
};
use color_eyre::eyre::eyre;
use directories::BaseDirs;
use futures::StreamExt;
use nanoid::nanoid;
use std::{
    env::{consts, temp_dir},
    fs::{canonicalize, create_dir_all, remove_dir_all},
    path::{Path, PathBuf},
};
use tokio::task::JoinHandle;

/// The fact sheet fills its page, while updates use the stylesheet margins.
#[derive(strum::EnumIs)]
pub enum PdfMargins {
    /// No printable margins.
    FullBleed,
    /// Let the stylesheet choose margins.
    Stylesheet,
}
/// Active Chromium browser and temporary profile for one PDF batch.
pub struct PdfSession {
    browser: Browser,
    handler: JoinHandle<()>,
    page: Page,
    profile: PathBuf,
}
impl PdfSession {
    /// Return the browser page used for rendering.
    pub fn page(&self) -> &Page {
        &self.page
    }
    /// Close Chromium and remove its temporary profile.
    pub async fn close(self) {
        let Self {
            mut browser,
            handler,
            page,
            profile,
        } = self;
        let _close_page = page.close().await;
        let _close_browser = browser.close().await;
        let _wait_browser = browser.wait().await;
        let _close_handler = handler.await;
        let _remove_profile = remove_dir_all(profile);
    }
}
fn browser_cache_dir() -> ApiResult<PathBuf> {
    chromium_cache_dir()
        .ok_or_else(|| eyre!("Failed to resolve Chromium cache directory"))
        .and_then(|path| {
            create_dir_all(&path)
                .map(|_| path)
                .map_err(|why| eyre!("Failed to create Chromium cache directory — {why}"))
        })
}
/// Resolve an explicit or pinned Chromium executable.
pub async fn browser_executable(offline: bool, chrome_path: Option<&Path>) -> ApiResult<Option<PathBuf>> {
    match chrome_path {
        | Some(path) => configured_browser_executable(path).map(Some),
        | None => fallback_browser_executable(offline).await,
    }
}
fn browser_profile_dir() -> ApiResult<PathBuf> {
    let path = temp_dir().join(format!("{APPLICATION}-chromiumoxide-{}", nanoid!()));
    create_dir_all(&path)
        .map(|_| path)
        .map_err(|why| eyre!("Failed to create Chromium profile directory — {why}"))
}
fn cached_browser_executable(cache: PathBuf) -> Option<PathBuf> {
    let platform = (consts::OS, consts::ARCH);
    let (major, minor, build, patch) = CHROME_VERSION;
    let version = format!("{major}.{minor}.{build}.{patch}");
    let relative = match platform {
        | ("linux", "x86_64") => Some(format!("linux-{version}/chrome-linux64/chrome")),
        | ("macos", "x86_64") => Some(format!(
            "mac-{version}/chrome-mac-x64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing"
        )),
        | ("macos", "aarch64") => Some(format!(
            "mac_arm-{version}/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing"
        )),
        | ("windows", "x86") => Some(format!("win32-{version}/chrome-win32/chrome.exe")),
        | ("windows", "x86_64" | "aarch64") => Some(format!("win64-{version}/chrome-win64/chrome.exe")),
        | _ => None,
    };
    relative.map(|path| cache.join(path)).filter(|path| path.is_file())
}
fn chromium_cache_dir() -> Option<PathBuf> {
    BaseDirs::new().map(|dirs| dirs.cache_dir().join(APPLICATION).join("chromiumoxide"))
}
/// Validate and canonicalize an explicitly configured Chromium executable.
pub fn configured_browser_executable(path: &Path) -> ApiResult<PathBuf> {
    match path.is_file() {
        | true => canonicalize(path).map_err(|why| eyre!("Failed to resolve Chrome executable at {} — {why}", path.display())),
        | false => Err(eyre!("Chrome executable is not a file: {}", path.display())),
    }
}
async fn fallback_browser_executable(offline: bool) -> ApiResult<Option<PathBuf>> {
    let (major, minor, build, patch) = CHROME_VERSION;
    match browser_cache_dir() {
        | Ok(path) if offline => Ok(cached_browser_executable(path)),
        | Ok(path) => {
            let options = BrowserFetcherOptions::builder()
                .with_kind(BrowserKind::Chrome)
                .with_version(Version::exact(major, minor, build, patch))
                .with_path(path)
                .build();
            match options {
                | Ok(options) => BrowserFetcher::new(options)
                    .fetch()
                    .await
                    .map(|installation| Some(installation.executable_path))
                    .map_err(|why| eyre!("Failed to install Chromium — {why}")),
                | Err(why) => Err(eyre!("Failed to configure Chromium download — {why}")),
            }
        }
        | Err(why) => Err(why),
    }
}
/// Start one Chromium session under the configured or cached executable policy.
pub async fn initialize_page(offline: bool, chrome_path: Option<&Path>) -> ApiResult<PdfSession> {
    match browser_executable(offline, chrome_path).await {
        | Ok(executable) => match browser_profile_dir() {
            | Ok(profile) => {
                let builder = BrowserConfig::builder().user_data_dir(&profile).no_sandbox().arg("--disable-gpu");
                let config = match executable {
                    | Some(path) => builder.chrome_executable(path),
                    | None => builder,
                }
                .build();
                match config {
                    | Ok(config) => match Browser::launch(config).await {
                        | Ok((mut browser, mut handler)) => {
                            let handler = tokio::spawn(async move {
                                while let Some(message) = handler.next().await {
                                    if message.is_err() {
                                        break;
                                    }
                                }
                            });
                            match browser.new_page("about:blank").await {
                                | Ok(page) => Ok(PdfSession {
                                    browser,
                                    handler,
                                    page,
                                    profile,
                                }),
                                | Err(why) => {
                                    let _close_browser = browser.close().await;
                                    let _wait_browser = browser.wait().await;
                                    let _close_handler = handler.await;
                                    let _remove_profile = remove_dir_all(profile);
                                    Err(eyre!("Failed to create browser page — {why}"))
                                }
                            }
                        }
                        | Err(why) => {
                            let _remove_profile = remove_dir_all(profile);
                            Err(eyre!("Failed to launch Chromium — {why}"))
                        }
                    },
                    | Err(why) => {
                        let _remove_profile = remove_dir_all(profile);
                        Err(eyre!("Failed to configure Chromium — {why}"))
                    }
                }
            }
            | Err(why) => Err(why),
        },
        | Err(why) => Err(why),
    }
}
/// Render HTML to an exact PDF output path with one browser page.
pub async fn render_html_to_pdf(page: &Page, content: &str, output_path: &Path, margins: PdfMargins) -> ApiResult<()> {
    let margin = margins.is_full_bleed().then_some(0.0);
    match page.set_content(content).await {
        | Ok(_) => page
            .save_pdf(
                PrintToPdfParams {
                    margin_bottom: margin,
                    margin_left: margin,
                    margin_right: margin,
                    margin_top: margin,
                    prefer_css_page_size: Some(true),
                    print_background: Some(true),
                    ..PrintToPdfParams::default()
                },
                output_path,
            )
            .await
            .map_err(|why| eyre!("Failed to render PDF at {} — {why}", output_path.display()))
            .and_then(|_| match output_path.is_file() {
                | true => Ok(()),
                | false => Err(eyre!("Chromium did not create PDF at {}", output_path.display())),
            }),
        | Err(why) => Err(eyre!("Failed to render PDF HTML — {why}")),
    }
}
