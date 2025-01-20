use crate::schema::{Contact, Metadata, ProjectSections, Research, ResearchActivity, Sections};
use base64::engine::general_purpose;
use base64::prelude::*;
use derive_more::Display;
use fast_qr::convert::image::{ImageBuilder, ImageError};
use fast_qr::convert::{Builder, Shape};
use fast_qr::qr::QRBuilder;
use percy_dom::prelude::*;

const BASE_URL: &str = "https://fact-sheets.ornl.gov";

// https://www.olcf.ornl.gov/about-olcf/media-assets/
const TRANSPARENT_RGBA: [u8; 4] = [255, 255, 255, 0];
const ORNL_GREEN_RGBA: [u8; 4] = [0, 121, 52, 255];

fn encode_data_uri(data: &[u8], mime_type: MimeType) -> String {
    format!(
        "data:{};base64,{}",
        mime_type,
        general_purpose::STANDARD.encode(data)
    )
}
fn generate_font_faces() -> String {
    format!(
        r#"
        @font-face {{font-family: "myriad";src: url("{}") format("opentype");font-style: normal;font-weight: 300;}}
        @font-face {{font-family: "myriad-semi";src: url({}) format("opentype");font-style: normal;font-weight: 300;}}
        @font-face {{font-family: "proxima";src: url({}) format("opentype");font-weight: normal;font-style: normal;}}
        @font-face {{font-family: "proxima-bold";src: url({}) format("opentype");font-weight: bold;font-style: normal;}}
        @font-face {{font-family: "proxima-semi";src: url({}) format("opentype");font-weight: bold;font-style: normal;}}
        @font-face {{font-family: "proxima";src: url({}) format("opentype");font-weight: normal;font-style: italic;}}
        "#,
        encode_data_uri(
            include_bytes!("../../assets/fonts/myriad_pro_light.otf"),
            MimeType::Otf
        ),
        encode_data_uri(
            include_bytes!("../../assets/fonts/myriad_pro_semibold.otf"),
            MimeType::Otf
        ),
        encode_data_uri(
            include_bytes!("../../assets/fonts/proxima_nova_light.otf"),
            MimeType::Otf
        ),
        encode_data_uri(
            include_bytes!("../../assets/fonts/proxima_nova_bold.otf"),
            MimeType::Otf
        ),
        encode_data_uri(
            include_bytes!("../../assets/fonts/proxima_nova_semibold.otf"),
            MimeType::Otf
        ),
        encode_data_uri(
            include_bytes!("../../assets/fonts/proxima_nova_regular_italic.otf"),
            MimeType::Otf
        ),
    )
}
fn generate_qr_code(text: &str) -> Result<Vec<u8>, ImageError> {
    let code = match QRBuilder::new(text).build() {
        Ok(data) => data,
        Err(err) => panic!("Failed to generate QR code: {}", err),
    };
    ImageBuilder::default()
        .shape(Shape::RoundedSquare)
        .background_color(TRANSPARENT_RGBA)
        .module_color(ORNL_GREEN_RGBA)
        .margin(0)
        .fit_width(125)
        .to_bytes(&code)
}
/// See https://developer.mozilla.org/en-US/docs/Web/HTTP/MIME_types/Common_types
#[derive(Clone, Debug, Display)]
pub enum MimeType {
    #[display("image/jpeg")]
    Jpeg,
    #[display("image/png")]
    Png,
    #[display("font/otf")]
    Otf,
    #[display("image/svg+xml")]
    Svg,
}
pub struct FactSheetFooter {
    /// TODO: Convert to enum
    pub division: String,
}
pub struct FactSheetHead {
    pub title: String,
    pub description: String,
}
impl View for Contact {
    fn render(&self) -> VirtualNode {
        let Contact {
            first,
            last,
            title: role,
            email,
            phone,
            ..
        } = self;
        html! {
            <section id="contact">
                <div>
                    <span class="label">Contact</span>
                    <span class="spacer"> </span>
                    <span class="name">{ format!("{} {}", first, last) }</span>
                    <span class="spacer">|</span>
                    <span class="title">{ role }</span>
                    <span class="spacer">|</span>
                    <span class="email">{ email }</span>
                    <span class="spacer">|</span>
                    <span class="phone">{ phone }</span>
                </div>
            </section>
        }
    }
}
impl View for FactSheetFooter {
    fn render(&self) -> VirtualNode {
        let disclaimer = "Oak Ridge National Laboratory is managed by UT-Batelle LLC for the US Department of Energy";
        let style = format!(
            "background-image: url({});",
            encode_data_uri(
                include_bytes!("../../assets/images/fact-sheet/footer.jpg"),
                MimeType::Jpeg
            )
        );
        let ornl_logo = encode_data_uri(
            include_bytes!("../../assets/images/logo_ornl.svg"),
            MimeType::Svg,
        );
        let doe_logo = encode_data_uri(
            include_bytes!("../../assets/images/logo_doe_os.png"),
            MimeType::Png,
        );
        html! {
            <footer style={ style }>
                <div class="wrapper">
                    <div id="disclaimer">{ disclaimer }</div>
                    <div class="logo-wrapper">
                        <div><img src={ ornl_logo } id="logo-ornl"/></div>
                        <div><img src={ doe_logo } id="logo-doe-os"/></div>
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
    let Metadata { id, .. } = meta;
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
                encode_data_uri(
                    include_bytes!("../../assets/images/fact-sheet/header.jpg"),
                    MimeType::Jpeg
                )
            );
            // TODO: Read research data image and base64 encode
            let graphic = "";
            let qrcode = encode_data_uri(
                &generate_qr_code(&format!("{}/{}", BASE_URL, id)).unwrap(),
                MimeType::Png,
            );
            let logo = encode_data_uri(
                include_bytes!("../../assets/images/logo_leaf.svg"),
                MimeType::Svg,
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
                                <img id="logo" src={ logo } height="76px"/>
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
