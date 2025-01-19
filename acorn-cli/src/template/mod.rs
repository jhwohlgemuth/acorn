use crate::schema::{Contact, ProjectSections, Research, ResearchActivity, Sections};
use base64::engine::general_purpose;
use base64::prelude::*;
use derive_more::Display;
use percy_dom::prelude::*;

/// See https://developer.mozilla.org/en-US/docs/Web/HTTP/MIME_types/Common_types
#[derive(Clone, Debug, Display)]
enum MimeType {
    #[display("image/jpeg")]
    Jpeg,
    #[display("image/png")]
    Png,
    // #[display("font/otf")]
    // Otf,
    #[display("image/svg+xml")]
    Svg,
    // #[display("font/ttf")]
    // Ttf,
}
fn encode_data_uri(data: &[u8], mime_type: MimeType) -> String {
    format!(
        "data:{};base64,{}",
        mime_type,
        general_purpose::STANDARD.encode(data)
    )
}
pub struct FactSheetFooter {}
pub struct FactSheetHead {}
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
        html! {
            <footer style={ format!("background-image: url({});", encode_data_uri(include_bytes!("../../assets/images/fact-sheet/footer.jpg"), MimeType::Jpeg)) }>
                <div class="wrapper">
                    <div id="disclaimer">
                        <div>{ "Oak Ridge National Laboratory is managed by UT-Batelle LLC for the US Department of Energy" }</div>
                    </div>
                    <div class="logo-wrapper">
                        <div>
                            <img src={ encode_data_uri(include_bytes!("../../assets/images/logo_ornl.svg"), MimeType::Svg) } id="logo-ornl"/>
                        </div>
                        <div>
                            <img src={ encode_data_uri(include_bytes!("../../assets/images/logo_doe_os.png"), MimeType::Png) } id="logo-doe-os"/>
                        </div>
                        <div id="nss">{ "National Security Sciences" }</div>
                    </div>
                </div>
            </footer>
        }
    }
}
impl View for FactSheetHead {
    fn render(&self) -> VirtualNode {
        let css =
            String::from_utf8_lossy(include_bytes!("../../assets/fact-sheet.css")).into_owned();
        let font_faces = format!(
            r#"
            @font-face {{
                font-family: "myriad";
                src: url("{}") format("opentype");
                font-style: normal;
                font-weight: 300;
            }}
            @font-face {{
                font-family: "myriad-semi";
                src: url("fonts/myriad_pro_semibold.otf") format("opentype");
                font-style: normal;
                font-weight: 300;
            }}
            @font-face {{
                font-family: "proxima";
                src: url("fonts/proxima_nova_light.otf") format("opentype");
                font-weight: normal;
                font-style: normal;
            }}
            @font-face {{
                font-family: "proxima-bold";
                src: url("fonts/proxima_nova_bold.otf") format("opentype");
                font-weight: bold;
                font-style: normal;
            }}
            @font-face {{
                font-family: "proxima-semi";
                src: url("fonts/proxima_nova_semibold.otf") format("opentype");
                font-weight: bold;
                font-style: normal;
            }}
            @font-face {{
                font-family: "proxima";
                src: url("fonts/proxima_nova_regular_italic.otf") format("opentype");
                font-weight: normal;
                font-style: italic;
            }}
        "#,
            "fonts/myriad_pro_light.otf"
        );
        html! {
            <head>
                <meta charset="utf-8">
                <title>{ "Research Activity" }</title>
                <meta name="description" content="Research Activity Data">
                <style>
                    { font_faces }
                    { css }
                </style>
            </head>
        }
    }
}
// impl View for ResearchActivity {
//     fn render(&self) -> VirtualNode {

//     }
// }

pub fn factsheet(data: ResearchActivity) -> VirtualNode {
    let ResearchActivity {
        title, subtitle, ..
    } = data;
    let head = FactSheetHead {};
    let footer = FactSheetFooter {};
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
            let qrcode = "qrcode.png";
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
                        <ul><li>{ outcomes.into_iter().map(|i| i.to_string()).collect::<String>() }</li></ul>
                    </section>
                </main>
            };
            let aside = html! {
                <aside>
                    <div class="graphic">
                        <img src="{{ src }}"/>
                    </div>
                    <div class="research-focus">
                        <h3>{ "Research Focus" }</h3>
                        <p>{ focus }</p>
                        <ul><li>{ areas.into_iter().map(|i| i.to_string()).collect::<String>() }</li></ul>
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
                        <header style={ format!("background-image: url({});", encode_data_uri(include_bytes!("../../assets/images/fact-sheet/header.jpg"), MimeType::Jpeg)) }>
                            <div class="wrapper">
                                <img id="logo" src={ encode_data_uri(include_bytes!("../../assets/images/logo_leaf.svg"), MimeType::Svg) } height="76px"/>
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
