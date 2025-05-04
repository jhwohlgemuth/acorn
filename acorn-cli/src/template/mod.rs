use crate::util::{Size, Target, TargetLabel};
use acorn_lib::constants::{BASE_URL, COLOR_PRIMARY, COLOR_TRANSPARENT, DISCLAIMER};
use acorn_lib::schema::{
    ContactPoint, HighlightSections, Metadata, Notes, OrganizationSections, Other, ProjectSections, Research, ResearchActivity, Sections,
};
use acorn_lib::util::{Label, MimeType};
use data_encoding::BASE64;
use fast_qr::convert::image::{ImageBuilder, ImageError};
use fast_qr::convert::{Builder, Shape};
use fast_qr::qr::QRBuilder;
use percy_dom::prelude::*;
use rust_embed::Embed;
use tracing::error;

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
        match self.target {
            | Target::FactSheet => {
                let path = format!("fact-sheet/{}/footer.jpg", self.label.get_folder());
                let style = format!("background-image: url({});", DataUri::from_asset(&path));
                html! {
                    <footer style={ style }>
                        <div class="wrapper">
                            <div id="disclaimer">{ DISCLAIMER }</div>
                            <div class="logo-wrapper">
                                <div><img src={ DataUri::from_asset("logo_ornl.svg") } id="logo-ornl"/></div>
                                <div><img src={ DataUri::from_asset("logo_doe_os.png") } id="logo-doe-os"/></div>
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
                            <div><img src={ DataUri::from_asset("logo_doe_os_green.png") } id="logo-doe-os"/></div>
                            <div><img src={ DataUri::from_asset("logo_ornl_green.svg") } id="logo-ornl"/></div>
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
                &format!("\n@page {{ margin: 0; size: {}; }}\n", size)
            }
            | Target::Poster => "",
        };
        let styles = get_stylesheet(self.stylesheet.to_string());
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
pub fn get_html(data: &ResearchActivity, target: Option<Target>, size: Option<Size>) -> String {
    let ResearchActivity {
        meta,
        title,
        subtitle,
        contact,
        ..
    } = data;
    let ContactPoint { affiliation, .. } = contact;
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
            let folder = label.get_folder();
            let path = format!("{}/{}/header.jpg", target, folder);
            let header_style = format!("background-image: url({});", DataUri::from_asset(&path));
            let graphic = meta.clone().get_first_graphic_href();
            let caption = meta.clone().get_first_graphic_caption();
            let qrcode = DataUri::from_bytes(&generate_qr_code(&format!("{}/{}", BASE_URL, meta.identifier)).unwrap(), MimeType::Png);
            html! {
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
            let Metadata { doi, publication, .. } = &meta;
            let ContactPoint {
                given_name,
                family_name,
                affiliation,
                ..
            } = &data.contact;
            let label = match affiliation {
                | Some(affiliation) => TargetLabel::from_organization(affiliation),
                | None => TargetLabel::default(),
            };
            let target = target.unwrap_or_default();
            let head = HtmlHead {
                size: size.unwrap(),
                stylesheet: "highlight.css",
                target,
            };
            let footer = HtmlFooter { label, target };
            let graphic = meta.clone().get_first_graphic_href();
            let caption = &meta.clone().graphics.unwrap()[0].caption;
            let separator = match subtitle {
                | Some(value) if !value.is_empty() => ": ".to_string(),
                | _ => "".to_string(),
            };
            let contact_block = match &data.notes {
                | Some(Other::Unformatted(value)) => {
                    html!(
                        <div class="contact">
                            <div>{ value }</div>
                        </div>
                    )
                }
                | Some(Other::Formatted(Notes { managers, programs, .. })) => {
                    let maybe_managers = match managers {
                        | Some(value) if !value.is_empty() => {
                            html! {
                                <div>
                                    <span>{ "Managers" }</span>
                                    <span>{ value.join(", ") }</span>
                                </div>
                            }
                        }
                        | _ => html! { <div></div> },
                    };
                    let maybe_programs = match programs {
                        | Some(value) if !value.is_empty() => {
                            html! {
                                <div>
                                    <span>{ "Programs" }</span>
                                    <span>{ value.join(", ") }</span>
                                </div>
                            }
                        }
                        | _ => html! { <div></div> },
                    };
                    let maybe_publication = match publication {
                        | Some(value) if !value.is_empty() => {
                            html! {
                                <div>
                                    <span>{ "Publication" }</span>
                                    <span>{ value }</span>
                                </div>
                            }
                        }
                        | _ => html! { <div></div> },
                    };
                    html!(
                        <div class="contact">
                            <div>
                                <span>{ "PI" }</span>
                                <span>{ given_name }&nbsp;{ family_name }</span>
                            </div>
                            { maybe_programs }
                            { maybe_managers }
                            { maybe_publication }
                            <div>
                                <span>{ "DOI" }</span>
                                <span>{ doi.clone().unwrap() }</span>
                            </div>
                        </div>
                    )
                }
                | None => html!(<div></div>),
            };
            html! {
                <html>
                    { head }
                    <body>
                        <header>
                            <h1>
                                <span>{ title }{ separator }</span>
                                <span>{ subtitle.clone().unwrap_or_else(|| "".to_string()) }</span>
                            </h1>
                        </header>
                        <div class="main">
                            <main>
                                <section>
                                    <h3>{ "Scientific Achievement" }</h3>
                                    <p>{ achievement }</p>
                                </section>
                                <section>
                                    <h3>{ "Significance and Impact" }</h3>
                                    <p>{ impact }</p>
                                </section>
                                <section>
                                    <h3>{ "Technical Approach" }</h3>
                                    <ul><li>{ technical.join("</li><li>") }</li></ul>
                                </section>
                            </main>
                            <aside>
                                <div class="graphic"><img src={ graphic }/></div>
                                <div class="caption">{ caption }</div>
                                { contact_block }
                            </aside>
                        </div>
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
            let ContactPoint { affiliation, .. } = &data.contact;
            let label = match affiliation {
                | Some(affiliation) => TargetLabel::from_organization(affiliation),
                | None => TargetLabel::default(),
            };
            let target = target.unwrap_or_default();
            let head = HtmlHead {
                size: size.unwrap(),
                stylesheet: "organization.css",
                target,
            };
            let footer = HtmlFooter { label, target };
            let folder = label.get_folder();
            let path = format!("{}/{}/header.jpg", target, folder);
            let header_style = format!("background-image: url({});", DataUri::from_asset(&path));
            let graphic = meta.clone().get_first_graphic_href();
            let caption = meta.clone().get_first_graphic_caption();
            let qrcode = DataUri::from_bytes(&generate_qr_code(&format!("{}/{}", BASE_URL, meta.identifier)).unwrap(), MimeType::Png);
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
                    <div class="caption">{ caption }</div>
                    <div class="research-focus">
                        <h3>{ "Facilities" }</h3>
                        <ul><li>{ facilities.clone().unwrap_or_default().join("</li><li>") }</li></ul>
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
                                <div id="subtitle">{ subtitle.clone().unwrap_or_else(|| "".to_string()) }</div>
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
        .background_color(COLOR_TRANSPARENT)
        .module_color(COLOR_PRIMARY)
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
