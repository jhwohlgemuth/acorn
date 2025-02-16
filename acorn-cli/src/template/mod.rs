use acorn_lib::schema::{HighlightSections, OrganizationSections, ProjectSections, Research, ResearchActivity, Sections};
use acorn_lib::util::{cli, path_to_string, Label, MimeType};
use data_encoding::BASE64;
use fast_qr::convert::image::{ImageBuilder, ImageError};
use fast_qr::convert::{Builder, Shape};
use fast_qr::qr::QRBuilder;
use percy_dom::prelude::*;
use playwright::api::Page;
use rust_embed::Embed;
use std::fs::{create_dir_all, remove_file, File};
use std::path::{self, PathBuf};
use tracing::{debug, error};

const BASE_URL: &str = "https://fact-sheets.ornl.gov";
// https://www.olcf.ornl.gov/about-olcf/media-assets/
const RGBA_TRANSPARENT: [u8; 4] = [255, 255, 255, 0];
const RGBA_ORNL_GREEN: [u8; 4] = [0, 121, 52, 255];

struct DataUri {}
#[derive(Embed)]
#[folder = "assets/images/"]
struct Image;
#[derive(Embed)]
#[folder = "assets/fonts/"]
struct Font;
#[derive(Embed)]
#[folder = "assets/styles/"]
struct Style;
struct HtmlFooter {
    /// TODO: Convert to enum
    pub organization: String,
}
struct HtmlHead {
    pub target: cli::Target,
    pub size: cli::Size,
    pub title: String,
    pub description: String,
    pub stylesheet: String,
}
impl DataUri {
    fn from_bytes(data: &[u8], mime: MimeType) -> String {
        format!("data:{};base64,{}", mime, BASE64.encode(data))
    }
    fn from_asset(value: &str) -> String {
        let mime = MimeType::from_path(value);
        match mime {
            | MimeType::Otf => {
                let binding = Font::get(value).unwrap();
                let data = binding.data.as_ref();
                DataUri::from_bytes(data, mime)
            }
            | MimeType::Jpeg | MimeType::Png | MimeType::Svg => {
                let binding = Image::get(value).unwrap();
                let data = binding.data.as_ref();
                DataUri::from_bytes(data, mime)
            }
            | _ => unimplemented!(),
        }
    }
}
impl View for HtmlFooter {
    fn render(&self) -> VirtualNode {
        let disclaimer = "Oak Ridge National Laboratory is managed by UT-Batelle LLC for the US Department of Energy";
        let style = format!("background-image: url({});", DataUri::from_asset("fact-sheet/footer.jpg"),);
        html! {
            <footer style={ style }>
                <div class="wrapper">
                    <div id="disclaimer">{ disclaimer }</div>
                    <div class="logo-wrapper">
                        <div><img src={ DataUri::from_asset("logo_ornl.svg") } id="logo-ornl"/></div>
                        <div><img src={ DataUri::from_asset("logo_doe_os.png") } id="logo-doe-os"/></div>
                        <div id="division">{ &self.organization }</div>
                    </div>
                </div>
            </footer>
        }
    }
}
impl View for HtmlHead {
    fn render(&self) -> VirtualNode {
        let styles = get_stylesheet(self.stylesheet.clone());
        let page_css = match self.target {
            | cli::Target::FactSheet => "",
            | cli::Target::Highlight => {
                let size = match self.size {
                    | cli::Size::Standard => "10in 7.5in",
                    | cli::Size::Widescreen => "13.33in 7.5in",
                };
                &format!("@page {{ size: {}; margin: 0; }}", size)
            }
            | cli::Target::Poster => "",
        };
        html! {
            <head>
                <meta charset="utf-8">
                <title>{ &self.title }</title>
                <meta name="description" content={ &self.description }>
                <style>
                    { generate_font_faces() }
                    { page_css }
                    { styles }
                </style>
            </head>
        }
    }
}
pub async fn create_pdf(path: PathBuf, page: &Page, output: &Option<PathBuf>) {
    let data = match ResearchActivity::read(path.clone()) {
        | Ok(value) => value.format(Some(path.clone())),
        | Err(err) => {
            error!(path = path_to_string(path), "=> {} {err}", Label::fail());
            std::process::exit(exitcode::UNAVAILABLE);
        }
    };
    let content = get_html(&data);
    // NOTE: Create temporary index.html because rust-playwright does not support setting baseURL
    let index = path::absolute(path.parent().unwrap().join("index.html")).unwrap();
    if File::create(&index).is_ok() {
        page.goto_builder(&format!("file://{}", index.display())).goto().await.unwrap();
    };
    page.set_content_builder(&content).set_content().await.unwrap();
    match output {
        | Some(output_dir) => match create_dir_all(output_dir.clone()) {
            | Ok(_) => {
                let id = data.meta.id;
                let output_path = format!("{}/{}.pdf", output_dir.display(), id);
                debug!(path = output_path, "=> {} Output", Label::using());
                page.pdf_builder().path(output_path.into()).print_background(true).pdf().await.unwrap();
            }
            | Err(err) => {
                error!(
                    path = output_dir.clone().into_os_string().into_string().unwrap(),
                    "=> {} Create directory - {err}",
                    Label::fail()
                );
            }
        },
        | None => unreachable!(),
    };
    let _remove = remove_file(index);
}
pub fn get_html(data: &ResearchActivity) -> String {
    let ResearchActivity { meta, title, subtitle, .. } = data;
    let node = match &data.sections {
        | Sections::Project(ProjectSections {
            introduction,
            challenge,
            approach,
            outcomes,
            research,
            ..
        }) => {
            let Research { focus, areas, .. } = research;
            let head = HtmlHead {
                target: cli::Target::FactSheet,
                size: cli::Size::Standard,
                title: "Research Activity".to_string(),
                description: "Research Activity Data".to_string(),
                stylesheet: "project.css".to_string(),
            };
            let footer = HtmlFooter {
                organization: "National Security Sciences".to_string(),
            };
            let header_style = format!("background-image: url({});", DataUri::from_asset("fact-sheet/header.jpg"));
            let graphic = meta.clone().get_first_graphic_href();
            let qrcode = DataUri::from_bytes(&generate_qr_code(&format!("{}/{}", BASE_URL, meta.id)).unwrap(), MimeType::Png);
            let main = html! {
                <main>
                    <section>
                        <p>{ introduction }</p>
                    </section>
                    <section>
                        <h3>{ "Challenge" }</h3>
                        <p>{ challenge }</p>
                    </section>
                    <section>
                        <h3>{ "Approach" }</h3>
                        <p>{ approach }</p>
                    </section>
                    <section>
                        <h3>{ "Outcomes" }</h3>
                        <p>{ "Previous and expected future outcomes include" }</p>
                        <ul><li>{ outcomes.join("</li><li>") }</li></ul>
                    </section>
                </main>
            };
            let aside = html! {
                <aside>
                    <div class="graphic"><img src={ graphic }/></div>
                    <div class="research-focus">
                        <h3>{ "Research Focus" }</h3>
                        <p>{ focus }</p>
                        <ul><li>{ areas.join("</li><li>") }</li></ul>
                    </div>
                    <div class="qrcode">
                        <img src={ qrcode }/>
                    </div>
                </aside>
            };
            html! {
                <html>
                    { head }
                    <body>
                        <header style={ header_style }>
                            <div class="wrapper">
                                <img id="logo" src={ DataUri::from_asset("logo_leaf.svg") } height="76px"/>
                                <div id="ornl-header">{ "Oak Ridge National Laboratory" }</div>
                                <div id="title">{ title }</div>
                                <div id="subtitle">{ subtitle }</div>
                            </div>
                        </header>
                        <div class="main">
                            { main }
                            { aside }
                        </div>
                        { data.clone().contact }
                        { footer }
                    </body>
                </html>
            }
        }
        | Sections::Highlight(HighlightSections {
            achievement,
            impact,
            technical,
        }) => {
            let head = HtmlHead {
                target: cli::Target::Highlight,
                size: cli::Size::Standard,
                title: "Highlight".to_string(),
                description: "Research Activity Data for Highlight".to_string(),
                stylesheet: "highlight.css".to_string(),
            };
            let footer = HtmlFooter {
                organization: "National Security Sciences".to_string(),
            };
            let header_style = format!("background-image: url({});", DataUri::from_asset("fact-sheet/header.jpg"));
            let graphic = meta.clone().get_first_graphic_href();
            let main = html! {
                <main>
                    <section>
                        <p>{ achievement }</p>
                    </section>
                    <section>
                        <p>{ impact }</p>
                    </section>
                    <section>
                        <h3>{ "Technical Approaches" }</h3>
                        <ul><li>{ technical.join("</li><li>") }</li></ul>
                    </section>
                </main>
            };
            let aside = html! {
                <aside>
                    <div class="graphic"><img src={ graphic }/></div>
                </aside>
            };
            html! {
                <html>
                    { head }
                    <body>
                        <header style={ header_style }>
                            <div class="wrapper">
                                <img id="logo" src={ DataUri::from_asset("logo_leaf.svg") } height="76px"/>
                                <div id="ornl-header">{ "Oak Ridge National Laboratory" }</div>
                                <div id="title">{ title }</div>
                                <div id="subtitle">{ subtitle }</div>
                            </div>
                        </header>
                        <div class="main">
                            { main }
                            { aside }
                        </div>
                        { data.clone().contact }
                        { footer }
                    </body>
                </html>
            }
        }
        | Sections::Organization(OrganizationSections {
            mission,
            capabilities,
            impact,
            facilities,
        }) => {
            let head = HtmlHead {
                target: cli::Target::FactSheet,
                size: cli::Size::Standard,
                title: "Research Activity".to_string(),
                description: "Research Activity Data".to_string(),
                stylesheet: "organization.css".to_string(),
            };
            let footer = HtmlFooter {
                organization: "National Security Sciences".to_string(),
            };
            let header_style = format!("background-image: url({});", DataUri::from_asset("fact-sheet/header.jpg"));
            let graphic = meta.clone().get_first_graphic_href();
            let qrcode = DataUri::from_bytes(&generate_qr_code(&format!("{}/{}", BASE_URL, meta.id)).unwrap(), MimeType::Png);
            let main = html! {
                <main>
                    <section>
                        <p>{ mission }</p>
                    </section>
                    <section>
                        <h3>{ "Capabilities" }</h3>
                        <ul><li>{ capabilities.join("</li><li>") }</li></ul>
                    </section>
                    <section>
                        <h3>{ "Impact" }</h3>
                        <ul><li>{ impact.join("</li><li>") }</li></ul>
                    </section>
                </main>
            };
            let aside = html! {
                <aside>
                    <div class="graphic"><img src={ graphic }/></div>
                    <div class="research-focus">
                        <h3>{ "Facilities" }</h3>
                        <ul><li>{ facilities.join("</li><li>") }</li></ul>
                    </div>
                    <div class="qrcode">
                        <img src={ qrcode }/>
                    </div>
                </aside>
            };
            html! {
                <html>
                    { head }
                    <body>
                        <header style={ header_style }>
                            <div class="wrapper">
                                <img id="logo" src={ DataUri::from_asset("logo_leaf.svg") } height="76px"/>
                                <div id="ornl-header">{ "Oak Ridge National Laboratory" }</div>
                                <div id="title">{ title }</div>
                                <div id="subtitle">{ subtitle }</div>
                            </div>
                        </header>
                        <div class="main">
                            { main }
                            { aside }
                        </div>
                        { data.clone().contact }
                        { footer }
                    </body>
                </html>
            }
        }
    };
    node.to_string()
}
fn generate_font_faces() -> String {
    format!(
        r#"
        @font-face {{font-family:"myriad_pro_light";src:url("{}") format("opentype");font-style:normal;font-weight:300;}}
        @font-face {{font-family:"myriad_pro_semibold";src:url({}) format("opentype");font-style:normal;font-weight:300;}}
        @font-face {{font-family:"proxima";src:url({}) format("opentype");font-weight:normal;font-style:normal;}}
        @font-face {{font-family:"proxima_nova_bold";src: url({}) format("opentype");font-weight:bold;font-style:normal;}}
        @font-face {{font-family:"proxima_nova_semibold";src:url({}) format("opentype");font-weight:bold;font-style:normal;}}
        @font-face {{font-family:"proxima";src:url({}) format("opentype");font-weight:normal;font-style:italic;}}
        "#,
        DataUri::from_asset("myriad_pro_light.otf"),
        DataUri::from_asset("myriad_pro_semibold.otf"),
        DataUri::from_asset("proxima_nova_light.otf"),
        DataUri::from_asset("proxima_nova_bold.otf"),
        DataUri::from_asset("proxima_nova_semibold.otf"),
        DataUri::from_asset("proxima_nova_regular_italic.otf"),
    )
}
fn generate_qr_code(text: &str) -> Result<Vec<u8>, ImageError> {
    let code = match QRBuilder::new(text).build() {
        | Ok(data) => data,
        | Err(err) => panic!("Failed to generate QR code: {}", err),
    };
    ImageBuilder::default()
        .shape(Shape::RoundedSquare)
        .background_color(RGBA_TRANSPARENT)
        .module_color(RGBA_ORNL_GREEN)
        .margin(0)
        .fit_width(125)
        .to_bytes(&code)
}
pub fn get_stylesheet(file_name: String) -> String {
    let data = match Style::get(&file_name) {
        | Some(value) => value.data,
        | None => {
            error!(file_name, "=> {} Import Style asset", Label::fail());
            unimplemented!()
        }
    };
    String::from_utf8_lossy(data.as_ref()).into_owned()
}
