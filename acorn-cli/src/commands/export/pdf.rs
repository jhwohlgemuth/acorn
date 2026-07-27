//! PDF export helpers
//!
//! This module renders Research Activity data to HTML and uses Chromium to generate a PDF output file.
use crate::cli::CommandOptions;
use crate::template::Convert;
use acorn::io::{ApiResult, InputOutput};
use acorn::prelude::{absolute, create_dir_all, remove_dir_all, remove_file, ErrorKind, OpenOptions, PathBuf};
use acorn::schema::research_activity::ResearchActivity;
use acorn::util::{Label, StringConversion};
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::page::PrintToPdfParams;
use chromiumoxide::fetcher::{BrowserFetcher, BrowserFetcherOptions, BrowserKind, Version};
use chromiumoxide::Page;
use color_eyre::eyre::eyre;
use directories::BaseDirs;
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
pub async fn create(page: &Page, options: CommandOptions) -> ApiResult<()> {
    let CommandOptions { output, .. } = options;
    let validated = options
        .path
        .ok_or_else(|| eyre!("Missing path for PDF export"))
        .and_then(|path| -> ApiResult<(PathBuf, PathBuf, ResearchActivity)> {
            ResearchActivity::read(path.clone())
                .map_err(|why| {
                    error!(path = path.to_absolute_string(), "=> {} Read data for PDF export — {why}", Label::fail());
                    eyre!("Read data for PDF export — {why}")
                })
                .and_then(|data| {
                    let formatted = data.format_with(Some(path.clone()));
                    path.parent()
                        .map(|p| (p.to_path_buf(), formatted))
                        .ok_or_else(|| {
                            error!(
                                path = path.to_absolute_string(),
                                "=> {} Cannot resolve parent directory for PDF export input",
                                Label::fail()
                            );
                            eyre!("Cannot resolve parent directory for PDF export input")
                        })
                        .map(|(parent, formatted)| (path, parent, formatted))
                })
        })
        .and_then(|(path, parent, data)| data.to_html().map(|content| (path, parent, data, content.to_string())));
    match validated {
        | Ok((path, parent, data, content)) => {
            let index_path = absolute(parent.join("index.html"))
                .map_err(|why| {
                    error!(
                        path = path.to_absolute_string(),
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
                        error!(path = index.to_absolute_string(), "=> {} Create temporary index — {why}", Label::fail());
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
                                    debug!(path = output_path.to_absolute_string(), "=> {} Output", Label::using());
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
                                    .map_err(|why| eyre!("Failed to render PDF at {} — {why}", output_path.to_absolute_string()))
                                    .and_then(|_| match output_path.is_file() {
                                        | true => Ok(()),
                                        | false => Err(eyre!("Chromium did not create PDF at {}", output_path.to_absolute_string())),
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
pub(crate) async fn initialize_page() -> ApiResult<PdfSession> {
    match browser_cache_dir().and_then(|path| {
        BrowserFetcherOptions::builder()
            .with_kind(BrowserKind::Chrome)
            .with_version(Version::exact(151, 0, 7922, 47))
            .with_path(path)
            .build()
            .map_err(|why| eyre!("Failed to configure Chromium download — {why}"))
    }) {
        | Ok(options) => match BrowserFetcher::new(options).fetch().await {
            | Ok(installation) => match browser_profile_dir() {
                | Ok(profile) => match BrowserConfig::builder()
                    .chrome_executable(installation.executable_path)
                    .user_data_dir(&profile)
                    .no_sandbox()
                    .arg("--disable-gpu")
                    .build()
                {
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
                },
                | Err(why) => Err(why),
            },
            | Err(why) => Err(eyre!("Failed to install Chromium — {why}")),
        },
        | Err(why) => Err(why),
    }
}
fn browser_profile_dir() -> ApiResult<PathBuf> {
    let path = std::env::temp_dir().join(format!("acorn-chromiumoxide-{}", nanoid!()));
    create_dir_all(&path)
        .map(|_| path)
        .map_err(|why| eyre!("Failed to create Chromium profile directory — {why}"))
}
fn browser_cache_dir() -> ApiResult<PathBuf> {
    BaseDirs::new()
        .map(|dirs| dirs.cache_dir().join("acorn").join("chromiumoxide"))
        .ok_or_else(|| eyre!("Failed to resolve Chromium cache directory"))
        .and_then(|path| {
            create_dir_all(&path)
                .map(|_| path)
                .map_err(|why| eyre!("Failed to create Chromium cache directory — {why}"))
        })
}
