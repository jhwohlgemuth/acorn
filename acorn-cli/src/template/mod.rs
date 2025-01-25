use acorn_lib::schema::{ProjectSections, Research, ResearchActivity, Sections};
use acorn_lib::util::get_extension;
use base64::engine::general_purpose;
use base64::prelude::*;
use derive_more::Display;
use fast_qr::convert::image::{ImageBuilder, ImageError};
use fast_qr::convert::{Builder, Shape};
use fast_qr::qr::QRBuilder;
use percy_dom::prelude::*;
use rust_embed::Embed;
use std::path::Path;

const BASE_URL: &str = "https://fact-sheets.ornl.gov";

// https://www.olcf.ornl.gov/about-olcf/media-assets/
const RGBA_TRANSPARENT: [u8; 4] = [255, 255, 255, 0];
const RGBA_ORNL_GREEN: [u8; 4] = [0, 121, 52, 255];

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
        Ok(data) => data,
        Err(err) => panic!("Failed to generate QR code: {}", err),
    };
    ImageBuilder::default()
        .shape(Shape::RoundedSquare)
        .background_color(RGBA_TRANSPARENT)
        .module_color(RGBA_ORNL_GREEN)
        .margin(0)
        .fit_width(125)
        .to_bytes(&code)
}
struct DataUri {}
#[derive(Embed)]
#[folder = "assets/images/"]
struct Image;
#[derive(Embed)]
#[folder = "assets/fonts/"]
struct Font;
/// See https://developer.mozilla.org/en-US/docs/Web/HTTP/MIME_types/Common_types
#[derive(Clone, Debug, Display)]
pub enum MimeType {
    #[display("font/otf")]
    Otf,
    #[display("image/jpeg")]
    Jpeg,
    #[display("image/png")]
    Png,
    #[display("image/svg+xml")]
    Svg,
}
impl MimeType {
    fn from_str(value: &str) -> MimeType {
        let name = &value.to_lowercase();
        match get_extension(Path::new(name)).as_str() {
            "jpg" | "jpeg" => MimeType::Jpeg,
            "png" => MimeType::Png,
            "otf" => MimeType::Otf,
            "svg" => MimeType::Svg,
            _ => unimplemented!("Unsupported MIME type"),
        }
    }
}
pub struct FactSheetFooter {
    /// TODO: Convert to enum
    pub division: String,
}
pub struct FactSheetHead {
    pub title: String,
    pub description: String,
}
impl DataUri {
    fn from_bytes(data: &[u8], mime: MimeType) -> String {
        format!(
            "data:{};base64,{}",
            mime,
            general_purpose::STANDARD.encode(data)
        )
    }
    fn from_asset(name: &str) -> String {
        let mime = MimeType::from_str(name);
        match mime {
            MimeType::Otf => {
                let binding = Font::get(name).unwrap();
                let data = binding.data.as_ref();
                DataUri::from_bytes(data, mime)
            }
            MimeType::Jpeg | MimeType::Png | MimeType::Svg => {
                let binding = Image::get(name).unwrap();
                let data = binding.data.as_ref();
                DataUri::from_bytes(data, mime)
            }
        }
    }
}
impl View for FactSheetFooter {
    fn render(&self) -> VirtualNode {
        let disclaimer = "Oak Ridge National Laboratory is managed by UT-Batelle LLC for the US Department of Energy";
        let style = format!(
            "background-image: url({});",
            DataUri::from_asset("fact-sheet/footer.jpg"),
        );
        html! {
            <footer style={ style }>
                <div class="wrapper">
                    <div id="disclaimer">{ disclaimer }</div>
                    <div class="logo-wrapper">
                        <div><img src={ DataUri::from_asset("logo_ornl.svg") } id="logo-ornl"/></div>
                        <div><img src={ DataUri::from_asset("logo_doe_os.png") } id="logo-doe-os"/></div>
                        <div id="division">{ &self.division }</div>
                    </div>
                </div>
            </footer>
        }
    }
}
impl View for FactSheetHead {
    fn render(&self) -> VirtualNode {
        html! {
            <head>
                <meta charset="utf-8">
                <title>{ &self.title }</title>
                <meta name="description" content={ &self.description }>
                <style>
                    { generate_font_faces() }
                    { String::from_utf8_lossy(include_bytes!("../../assets/fact-sheet.css")).into_owned() }
                </style>
            </head>
        }
    }
}

pub fn factsheet(data: ResearchActivity) -> VirtualNode {
    let ResearchActivity {
        meta,
        title,
        subtitle,
        ..
    } = data;
    let head = FactSheetHead {
        title: "Research Activity".to_string(),
        description: "Research Activity Data".to_string(),
    };
    let footer = FactSheetFooter {
        division: "National Security Sciences".to_string(),
    };
    match data.sections {
        Sections::Project(ProjectSections {
            introduction,
            challenge,
            approach,
            outcomes,
            research,
            ..
        }) => {
            let Research { focus, areas, .. } = research;
            let header_style = format!(
                "background-image: url({});",
                DataUri::from_asset("fact-sheet/header.jpg")
            );
            let graphic = meta.clone().get_first_graphic_href();
            let qrcode = DataUri::from_bytes(
                &generate_qr_code(&format!("{}/{}", BASE_URL, meta.id)).unwrap(),
                MimeType::Png,
            );
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
                        { data.contact }
                        { footer }
                    </body>
                </html>
            }
        }
        Sections::Highlight(_) => unimplemented!("Highlight not implemented"),
        Sections::Organization(_) => unimplemented!("Organization not implemented"),
    }
}
