//! PDF export helpers
//!
//! This module renders Research Activity data to HTML and uses Chromium to generate a PDF output file.
use crate::cli::CommandOptions;
use crate::io::chromium_cache_dir;
use crate::template::Convert;
use acorn::io::chart::ChartOptions;
use acorn::io::{ApiResult, InputOutput};
use acorn::prelude::{absolute, canonicalize, consts, create_dir_all, remove_dir_all, remove_file, temp_dir, ErrorKind, OpenOptions, Path, PathBuf};
use acorn::schema::research_activity::ResearchActivity;
use acorn::util::constants::app::CHROME_VERSION;
use acorn::util::{Label, StringConversion};
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::page::PrintToPdfParams;
use chromiumoxide::fetcher::{BrowserFetcher, BrowserFetcherOptions, BrowserKind, Version};
use chromiumoxide::Page;
use color_eyre::eyre::eyre;
use futures::StreamExt;
use nanoid::nanoid;
use tokio::task::JoinHandle;
use tracing::{debug, error};

pub(crate) struct PdfSession {
    browser: Browser,
    handler: JoinHandle<()>,
    page: Page,
    profile: PathBuf,
}
impl PdfSession {
    pub(crate) fn page(&self) -> &Page {
        &self.page
    }
    pub(crate) async fn close(self) {
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
/// Create PDF document from Research Activity data
pub async fn create(page: &Page, options: CommandOptions, chart_options: ChartOptions) -> ApiResult<()> {
    let CommandOptions { output, .. } = options;
    let validated =
        options
            .path
            .ok_or_else(|| eyre!("Missing path for PDF export"))
            .and_then(|path| -> ApiResult<(PathBuf, PathBuf, ResearchActivity)> {
                ResearchActivity::read(path.clone())
                    .map_err(|why| {
                        error!(path = path.to_absolute_path(), "=> {} Read data for PDF export — {why}", Label::fail());
                        eyre!("Read data for PDF export — {why}")
                    })
                    .and_then(|data| {
                        let formatted = data.format_with(Some(path.clone()));
                        path.parent()
                            .map(|p| (p.to_path_buf(), formatted))
                            .ok_or_else(|| {
                                error!(
                                    path = path.to_absolute_path(),
                                    "=> {} Cannot resolve parent directory for PDF export input",
                                    Label::fail()
                                );
                                eyre!("Cannot resolve parent directory for PDF export input")
                            })
                            .map(|(parent, formatted)| (path, parent, formatted))
                    })
            });
    let rendered = validated.and_then(|(path, parent, data)| {
        chart_options.render(data.aspect.as_ref()).and_then(|aspect_chart| {
            data.to_html(aspect_chart.as_deref())
                .map(|content| (path, parent, data, content.to_string()))
        })
    });
    match rendered {
        | Ok((path, parent, data, content)) => {
            let index_path = absolute(parent.join("index.html"))
                .map_err(|why| {
                    error!(
                        path = path.to_absolute_path(),
                        "=> {} Resolve temporary index path — {why}",
                        Label::fail()
                    );
                    why
                })
                .ok();
            if let Some(ref index) = index_path {
                let can_navigate = match OpenOptions::new().write(true).create_new(true).open(index) {
                    | Ok(_) => true,
                    | Err(why) if why.kind() == ErrorKind::AlreadyExists => true,
                    | Err(why) => {
                        error!(path = index.to_absolute_path(), "=> {} Create temporary index — {why}", Label::fail());
                        false
                    }
                };
                if can_navigate {
                    let _goto = page.goto(format!("file://{}", index.display())).await;
                }
            }
            match page.set_content(&content).await {
                | Ok(_) => match output {
                    | Some(output_dir) => match create_dir_all(output_dir.clone()) {
                        | Ok(_) => {
                            let id = data.meta.identifier;
                            match absolute(output_dir.join(format!("{id}.pdf"))) {
                                | Ok(output_path) => {
                                    debug!(path = output_path.to_absolute_path(), "=> {} Output", Label::using());
                                    page.save_pdf(
                                        PrintToPdfParams {
                                            margin_bottom: Some(0.0),
                                            margin_left: Some(0.0),
                                            margin_right: Some(0.0),
                                            margin_top: Some(0.0),
                                            prefer_css_page_size: Some(true),
                                            print_background: Some(true),
                                            ..PrintToPdfParams::default()
                                        },
                                        &output_path,
                                    )
                                    .await
                                    .map_err(|why| eyre!("Failed to render PDF at {} — {why}", output_path.to_absolute_path()))
                                    .and_then(|_| match output_path.is_file() {
                                        | true => Ok(()),
                                        | false => Err(eyre!("Chromium did not create PDF at {}", output_path.to_absolute_path())),
                                    })
                                }
                                | Err(why) => Err(eyre!("Failed to resolve PDF output path — {why}")),
                            }
                        }
                        | Err(err) => {
                            error!("=> {} Create directory — {err}", Label::fail());
                            Err(eyre!("Failed to create output directory for PDF — {err}"))
                        }
                    },
                    | None => Err(eyre!("Missing output directory for PDF export")),
                },
                | Err(why) => Err(eyre!("Failed to render PDF HTML — {why}")),
            }
            .inspect(|_| {
                if let Some(index) = &index_path {
                    let _remove = remove_file(index);
                }
            })
        }
        | Err(e) => Err(e),
    }
}
pub(crate) async fn initialize_page(offline: bool, chrome_path: Option<&Path>) -> ApiResult<PdfSession> {
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
fn browser_profile_dir() -> ApiResult<PathBuf> {
    let path = temp_dir().join(format!("acorn-chromiumoxide-{}", nanoid!()));
    create_dir_all(&path)
        .map(|_| path)
        .map_err(|why| eyre!("Failed to create Chromium profile directory — {why}"))
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
pub(crate) async fn browser_executable(offline: bool, chrome_path: Option<&Path>) -> ApiResult<Option<PathBuf>> {
    match chrome_path {
        | Some(path) => configured_browser_executable(path).map(Some),
        | None => fallback_browser_executable(offline).await,
    }
}
pub(crate) fn configured_browser_executable(path: &Path) -> ApiResult<PathBuf> {
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
