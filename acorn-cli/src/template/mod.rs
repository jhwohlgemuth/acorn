use crate::cli::arguments::{Size, Target};
use acorn_lib::constants::{BASE_URL, COLOR_PRIMARY, COLOR_TRANSPARENT, DISCLAIMER};
use acorn_lib::schema::{ContactPoint, Research, ResearchActivity, Sections};
use acorn_lib::util::{Label, MimeType};
use data_encoding::BASE64;
use derive_more::Display;
use fast_qr::convert::image::{ImageBuilder, ImageError};
use fast_qr::convert::{Builder, Shape};
use fast_qr::qr::QRBuilder;
use percy_dom::prelude::{html, IterableNodes, View, VirtualNode};
use rust_embed::{Embed, EmbeddedFile};
use tracing::error;

pub trait Convert {
    fn to_html(&self, target: Option<Target>, size: Option<Size>) -> VirtualNode;
}
/// Target artifact branding available when exporting research activity data using acorn
///
/// Used primarily by ACORN CLI
#[derive(Default, Clone, Copy, Debug, Display)]
pub enum TargetLabel {
    /// National Security Sciences Directorate
    ///
    /// See <https://www.ornl.gov/science-area/national-security>
    #[default]
    #[display("National Security Sciences")]
    Nssd,
    /// Biological and Environmental Systems Sciences Directorate
    ///
    /// See <https://www.ornl.gov/directorate/bessd>
    #[display("Biological and Environmental Systems Sciences")]
    Bessd,
    /// ORNL Water Power Program
    ///
    /// See <https://www.ornl.gov/waterpower>
    #[display("Water Power Program")]
    Wpp,
    /// General ORNL Branding
    #[display("Solving Big Problems")]
    Ornl,
}
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
    pub label: TargetLabel,
    pub target: Target,
}
struct HtmlHead<'a> {
    pub size: Size,
    pub stylesheet: &'a str,
    pub target: Target,
}
impl DataUri {
    fn from_bytes(data: &[u8], mime: MimeType) -> String {
        format!("data:{};base64,{}", mime, BASE64.encode(data))
    }
    fn from_asset(value: &str) -> String {
        let mime = MimeType::from_string(value);
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
impl Style {
    pub fn to_string(file_name: String) -> String {
        let data = match Style::get(&file_name) {
            | Some(EmbeddedFile { data, .. }) => data,
            | None => {
                error!(file_name, "=> {} Import Style asset", Label::fail());
                unimplemented!()
            }
        };
        String::from_utf8_lossy(data.as_ref()).into_owned()
    }
}
impl TargetLabel {
    /// Returns a string representing the folder name for the given TargetLabel
    pub fn folder(self) -> String {
        match self {
            | TargetLabel::Bessd => "bessd".to_string(),
            | TargetLabel::Wpp => "wpp".to_string(),
            | TargetLabel::Nssd => "nssd".to_string(),
            | TargetLabel::Ornl => TargetLabel::default().folder().to_owned(),
        }
    }
    /// Returns a TargetLabel based on the given organization name
    pub fn from_organization(name: &str) -> Self {
        match name {
            | "Biological and Environmental Systems Science Directorate" => TargetLabel::Bessd,
            | "National Security Sciences Division" => TargetLabel::Nssd,
            | "Oak Ridge National Laboratory" => TargetLabel::Ornl,
            | "Water Power Program" => TargetLabel::Wpp,
            | _ => TargetLabel::default(),
        }
    }
}
impl View for HtmlFooter {
    fn render(&self) -> VirtualNode {
        match self.target {
            | Target::FactSheet => {
                let path = format!("fact-sheet/{}/footer.jpg", self.label.folder());
                let style = format!("background-image: url({});", DataUri::from_asset(&path));
                html! {
                    <footer style={ style }>
                        <div class="wrapper">
                            <div id="disclaimer">{ DISCLAIMER }</div>
                            <div class="logo-wrapper">
                                <div><img src={ DataUri::from_asset("logo_ornl_white.svg") } id="logo-ornl"/></div>
                                <div><img src={ DataUri::from_asset("logo_doe_white.png") } id="logo-doe"/></div>
                                <div id="organization">{ &self.label.to_string() }</div>
                            </div>
                        </div>
                    </footer>
                }
            }
            | Target::Highlight => html! {
                <footer>
                    <div class="wrapper">
                        <div class="logo-wrapper">
                            <div><img src={ DataUri::from_asset("logo_doe_ocolor.png") } id="logo-doe"/></div>
                            <div><img src={ DataUri::from_asset("logo_ornl_color.svg") } id="logo-ornl"/></div>
                        </div>
                    </div>
                </footer>
            },
            | Target::Poster => html! {
                <footer></footer>
            },
        }
    }
}
impl View for HtmlHead<'_> {
    fn render(&self) -> VirtualNode {
        let page_css = match self.target {
            | Target::FactSheet => "letter portrait",
            | Target::Highlight => {
                let size = match self.size {
                    | Size::Standard => "10in 7.5in",
                    | Size::Widescreen => "13.33in 7.5in",
                };
                &format!("\n@page {{ margin: 0; size: {size}; }}\n")
            }
            | Target::Poster => "",
        };
        let styles = Style::to_string(self.stylesheet.to_string());
        let style = html! {
            <style>
                { generate_font_faces() }
                { page_css }
                { styles }
            </style>
        };
        html! {
            <head>
                <meta charset="utf-8">
                <title>{ "Research Activity Data" }</title>
                <meta name="description" content={ "Research Activity Data" }>
                { style }
            </head>
        }
    }
}
impl Convert for ResearchActivity {
    fn to_html(&self, target: Option<Target>, size: Option<Size>) -> VirtualNode {
        let ResearchActivity {
            meta,
            title,
            subtitle,
            contact,
            ..
        } = self;
        let ContactPoint { affiliation, .. } = contact;
        let Sections {
            mission,
            challenge,
            approach,
            impact,
            research,
            ..
        } = &self.sections;
        let Research { focus, areas, .. } = research;
        let label = match affiliation {
            | Some(affiliation) => TargetLabel::from_organization(affiliation),
            | None => TargetLabel::default(),
        };
        let target = target.unwrap_or_default();
        let head = HtmlHead {
            size: size.unwrap(),
            stylesheet: "project.css",
            target,
        };
        let footer = HtmlFooter { label, target };
        let folder = label.folder();
        let path = format!("{target}/{folder}/header.jpg");
        let header_style = format!("background-image: url({});", DataUri::from_asset(&path));
        let graphic = meta.clone().first_image_content_url();
        let caption = meta.clone().first_image_caption();
        let qrcode = DataUri::from_bytes(&generate_qr_code(&format!("{}/{}", BASE_URL, meta.identifier)).unwrap(), MimeType::Png);
        let node = html! {
            <html>
                { head }
                <body>
                    <header style={ header_style }>
                        <div class="wrapper">
                            <img id="logo" src={ DataUri::from_asset("logo_leaf.svg") } height="76px"/>
                            <div id="ornl-header">{ "Oak Ridge National Laboratory" }</div>
                            <div id="title">{ title }</div>
                            <div id="subtitle">{ subtitle.clone().unwrap_or_else(|| "".to_string()) }</div>
                        </div>
                    </header>
                    <div class="main">
                        <main>
                            <section>
                                <p>{ mission }</p>
                            </section>
                            <section>
                                <h3>{ "Challenge" }</h3>
                                <p>{ challenge }</p>
                            </section>
                            <section>
                                <h3>{ "Approach" }</h3>
                                <ul><li>{ approach.join("</li><li>") }</li></ul>
                            </section>
                            <section>
                                <h3>{ "Impact" }</h3>
                                <ul><li>{ impact.join("</li><li>") }</li></ul>
                            </section>
                        </main>
                        <aside>
                            <div class="graphic"><img src={ graphic }/></div>
                            <div class="caption">{ caption }</div>
                            <div class="research-focus">
                                <h3>{ "Research Focus" }</h3>
                                <p>{ focus }</p>
                                <ul><li>{ areas.join("</li><li>") }</li></ul>
                            </div>
                            <div class="qrcode">
                                <img src={ qrcode }/>
                            </div>
                        </aside>
                    </div>
                    { self.clone().contact }
                    { footer }
                </body>
            </html>
        };
        node
    }
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
        | Err(err) => panic!("Failed to generate QR code: {err}"),
    };
    ImageBuilder::default()
        .shape(Shape::RoundedSquare)
        .background_color(COLOR_TRANSPARENT)
        .module_color(COLOR_PRIMARY)
        .margin(0)
        .fit_width(125)
        .to_bytes(&code)
}
