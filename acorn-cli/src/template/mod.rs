use acorn::io::ApiResult;
use acorn::schema::research_activity::{Research, ResearchActivity, Sections};
use acorn::schema::ContactPoint;
use acorn::util::constants::app::{BASE_URL, COLOR_PRIMARY, COLOR_TRANSPARENT, DISCLAIMER};
use acorn::util::{Label, MimeType};
use color_eyre::eyre::eyre;
use core::str::from_utf8;
use data_encoding::BASE64;
use derive_more::Display;
use fast_qr::convert::image::ImageBuilder;
use fast_qr::convert::{Builder, Shape};
use fast_qr::qr::QRBuilder;
use percy_dom::prelude::{html, IterableNodes, VirtualNode};
use rust_embed::{Embed, EmbeddedFile};
use tracing::error;

pub trait Convert {
    fn to_html(&self, aspect_chart: Option<&[u8]>) -> ApiResult<VirtualNode>;
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
}
struct HtmlHead<'a> {
    pub stylesheet: &'a str,
}
impl DataUri {
    fn from_bytes(data: &[u8], mime: MimeType) -> String {
        format!("data:{};base64,{}", mime, BASE64.encode(data))
    }
    fn from_asset(value: &str) -> ApiResult<String> {
        let mime = MimeType::from(value);
        match mime {
            | MimeType::Otf | MimeType::Ttf => match Font::get(value) {
                | Some(binding) => {
                    let data = binding.data.as_ref();
                    Ok(DataUri::from_bytes(data, mime))
                }
                | None => Err(eyre!("Missing embedded font asset: {value}")),
            },
            | MimeType::Jpeg | MimeType::Png | MimeType::Svg => match Image::get(value) {
                | Some(binding) => {
                    let data = binding.data.as_ref();
                    Ok(DataUri::from_bytes(data, mime))
                }
                | None => Err(eyre!("Missing embedded image asset: {value}")),
            },
            | _ => Err(eyre!("Unsupported embedded asset MIME type: {value}")),
        }
    }
}
impl Style {
    pub fn to_string(file_name: String) -> ApiResult<String> {
        Style::get(&file_name)
            .map(|EmbeddedFile { data, .. }| data)
            .ok_or_else(|| eyre!("Missing embedded style asset: {file_name}"))
            .and_then(|data| {
                from_utf8(data.as_ref())
                    .map(str::to_string)
                    .map_err(|why| eyre!("Invalid UTF-8 in embedded style asset {file_name}: {why}"))
            })
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
impl HtmlFooter {
    fn render(&self) -> ApiResult<VirtualNode> {
        let path = format!("fact-sheet/{}/footer.jpg", self.label.folder());
        DataUri::from_asset(&path)
            .and_then(|footer| DataUri::from_asset("logo_ornl_white.svg").map(|ornl| (footer, ornl)))
            .and_then(|(footer, ornl)| DataUri::from_asset("logo_doe_white.png").map(|doe| (footer, ornl, doe)))
            .map(|(footer, ornl, doe)| {
                let style = format!("background-image: url({footer});");
                html! {
                    <footer style={ style }>
                        <div class="wrapper">
                            <div id="disclaimer">{ DISCLAIMER }</div>
                            <div class="logo-wrapper">
                                <div><img src={ ornl } id="logo-ornl"/></div>
                                <div><img src={ doe } id="logo-doe"/></div>
                                <div id="organization">{ &self.label.to_string() }</div>
                            </div>
                        </div>
                    </footer>
                }
            })
    }
}
impl HtmlHead<'_> {
    fn render(&self) -> ApiResult<VirtualNode> {
        let page_css = "letter portrait";
        Style::to_string(self.stylesheet.to_string())
            .and_then(|styles| generate_font_faces().map(|fonts| (styles, fonts)))
            .map(|(styles, fonts)| {
                let style = html! {
                    <style>
                        { fonts }
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
            })
    }
}
impl Convert for ResearchActivity {
    fn to_html(&self, aspect_chart: Option<&[u8]>) -> ApiResult<VirtualNode> {
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
        let head = HtmlHead { stylesheet: "project.css" };
        let footer = HtmlFooter { label };
        let folder = label.folder();
        let path = format!("fact-sheet/{folder}/header.jpg");
        let graphic = meta.clone().first_image_content_url();
        let caption = meta.clone().first_image_caption();
        let aspect_chart = aspect_chart.map(|bytes| DataUri::from_bytes(bytes, MimeType::Svg));
        let qrcode = match generate_qr_code(&format!("{}/{}", BASE_URL, meta.identifier)) {
            | Some(bytes) => DataUri::from_bytes(&bytes, MimeType::Png),
            | None => {
                error!("=> {} Failed to generate QR code", Label::fail());
                String::new()
            }
        };
        head.render()
            .and_then(|head| footer.render().map(|footer| (head, footer)))
            .and_then(|(head, footer)| DataUri::from_asset(&path).map(|header| (head, footer, header)))
            .and_then(|(head, footer, header)| DataUri::from_asset("logo_leaf.svg").map(|logo| (head, footer, header, logo)))
            .map(|(head, footer, header, logo)| {
                let header_style = format!("background-image: url({header});");
                html! {
                    <html>
                        { head }
                        <body>
                            <header style={ header_style }>
                                <div class="wrapper">
                                    <img id="logo" src={ logo } height="76px"/>
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
                                    <div class="graphic">
                                        <img src={ graphic }/>
                                    </div>
                                    <div class="caption">{ caption }</div>
                                    <div class="research-focus">
                                        <h3>{ "Research Focus" }</h3>
                                        <p>{ focus }</p>
                                        <ul><li>{ areas.join("</li><li>") }</li></ul>
                                    </div>
                                    <div class="codes">
                                        {
                                            aspect_chart.map(|chart| html! {
                                                <img class="aspect-chart" src={ chart } alt="ASPECT rose chart"/>
                                            })
                                        }
                                        <img class="qrcode" src={ qrcode } alt="Research activity QR code"/>
                                    </div>
                                </aside>
                            </div>
                            { self.clone().contact }
                            { footer }
                        </body>
                    </html>
                }
            })
    }
}
fn generate_font_faces() -> ApiResult<String> {
    DataUri::from_asset("aptos.ttf")
        .and_then(|regular| DataUri::from_asset("aptos_bold.ttf").map(|bold| (regular, bold)))
        .and_then(|(regular, bold)| DataUri::from_asset("aptos_italic.ttf").map(|italic| (regular, bold, italic)))
        .map(|(regular, bold, italic)| {
            format!(
                r#"
        @font-face {{font-family:"aptos";src:url({}) format("truetype");font-style:normal;font-weight:400;}}
        @font-face {{font-family:"aptos";src:url({}) format("truetype");font-style:normal;font-weight:600;}}
        @font-face {{font-family:"aptos";src:url({}) format("truetype");font-style:normal;font-weight:700;}}
        @font-face {{font-family:"aptos";src:url({}) format("truetype");font-style:italic;font-weight:400;}}
        "#,
                regular, bold, bold, italic
            )
        })
}
fn generate_qr_code(text: &str) -> Option<Vec<u8>> {
    let code = match QRBuilder::new(text).build() {
        | Ok(data) => data,
        | Err(why) => {
            error!(?why, "=> {} Build QR code data", Label::fail());
            return None;
        }
    };
    match ImageBuilder::default()
        .shape(Shape::RoundedSquare)
        .background_color(COLOR_TRANSPARENT)
        .module_color(COLOR_PRIMARY)
        .margin(0)
        .fit_width(125)
        .to_bytes(&code)
    {
        | Ok(bytes) => Some(bytes),
        | Err(why) => {
            error!(?why, "=> {} Render QR code image", Label::fail());
            None
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]
    use super::{Convert, DataUri, Style};
    use acorn::schema::research_activity::ResearchActivity;

    #[test]
    fn test_required_embedded_assets_exist() {
        let assets = [
            "aptos.ttf",
            "aptos_bold.ttf",
            "aptos_italic.ttf",
            "fact-sheet/bessd/footer.jpg",
            "fact-sheet/bessd/header.jpg",
            "fact-sheet/nssd/footer.jpg",
            "fact-sheet/nssd/header.jpg",
            "fact-sheet/wpp/footer.jpg",
            "fact-sheet/wpp/header.jpg",
            "logo_doe_white.png",
            "logo_leaf.svg",
            "logo_ornl_white.svg",
        ];
        assert!(assets.into_iter().all(|asset| DataUri::from_asset(asset).is_ok()));
        assert!(Style::to_string("project.css".to_string()).is_ok());
    }
    #[test]
    fn test_missing_embedded_assets_return_errors() {
        assert!(DataUri::from_asset("missing.png").is_err());
        assert!(Style::to_string("missing.css".to_string()).is_err());
    }
    #[test]
    fn test_aspect_chart_precedes_qr_code_in_shared_container() {
        let activity: ResearchActivity = serde_json::from_str(include_str!("../../../tests/fixtures/data/highlight/hecate/index.json"))
            .expect("HECATE fixture should deserialize");
        let html = activity
            .to_html(Some(b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>"))
            .expect("fixture should render")
            .to_string();
        let codes = html.find("class=\"codes\"").expect("code container should render");
        let aspect = html.find("class=\"aspect-chart\"").expect("ASPECT chart should render");
        let qr = html.find("class=\"qrcode\"").expect("QR code should render");
        assert!(codes < aspect);
        assert!(aspect < qr);
    }
}
