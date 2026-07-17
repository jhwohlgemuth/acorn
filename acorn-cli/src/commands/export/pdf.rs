//! PDF export helpers
//!
//! This module renders Research Activity data to HTML and uses Playwright to
//! generate a PDF output file.
use crate::cli::CommandOptions;
use crate::template::Convert;
use acorn::io::{ApiResult, InputOutput};
use acorn::prelude::{absolute, create_dir_all, remove_file, ErrorKind, OpenOptions, PathBuf};
use acorn::schema::research_activity::ResearchActivity;
use acorn::util::{Label, StringConversion};
use color_eyre::eyre::eyre;
use playwright::api::{Browser, BrowserContext, Page};
use playwright::Playwright;
use tracing::{debug, error};

pub(crate) struct PdfSession {
    _playwright: Playwright,
    _browser: Browser,
    _context: BrowserContext,
    page: Page,
}
impl PdfSession {
    pub(crate) fn page(&self) -> &Page {
        &self.page
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
                    let _goto = page.goto_builder(&format!("file://{}", index.display())).goto().await;
                }
            }
            match page.set_content_builder(&content).set_content().await {
                | Ok(_) => match output {
                    | Some(output_dir) => match create_dir_all(output_dir.clone()) {
                        | Ok(_) => {
                            let id = data.meta.identifier;
                            match absolute(output_dir.join(format!("{id}.pdf"))) {
                                | Ok(output_path) => {
                                    debug!(path = output_path.to_absolute_string(), "=> {} Output", Label::using());
                                    page.pdf_builder()
                                        .prefer_css_page_size(true)
                                        .path(output_path.clone())
                                        .print_background(true)
                                        .pdf()
                                        .await
                                        .map_err(|why| eyre!("Failed to render PDF at {} — {why}", output_path.to_absolute_string()))
                                        .and_then(|_| match output_path.is_file() {
                                            | true => Ok(()),
                                            | false => Err(eyre!("Playwright did not create PDF at {}", output_path.to_absolute_string())),
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
    match Playwright::initialize()
        .await
        .map_err(|why| eyre!("Failed to initialize Playwright — {why}"))
        .and_then(install_chromium)
    {
        | Ok(playwright) => match playwright.chromium().launcher().headless(true).launch().await {
            | Ok(browser) => match browser.context_builder().build().await {
                | Ok(context) => match context.new_page().await {
                    | Ok(page) => Ok(PdfSession {
                        _playwright: playwright,
                        _browser: browser,
                        _context: context,
                        page,
                    }),
                    | Err(why) => Err(eyre!("Failed to create browser page — {why}")),
                },
                | Err(why) => Err(eyre!("Failed to create browser context — {why}")),
            },
            | Err(why) => Err(eyre!("Failed to launch Chromium — {why}")),
        },
        | Err(why) => Err(why),
    }
}
fn install_chromium(playwright: Playwright) -> ApiResult<Playwright> {
    playwright
        .install_chromium()
        .map_err(|why| eyre!("Failed to install Chromium for Playwright — {why}"))
        .map(|_| playwright)
}
