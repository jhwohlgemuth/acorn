use crate::schema::{Contact, ProjectSections, Research, ResearchActivity, Sections};
use percy_dom::prelude::*;

pub struct Footer {}
pub struct Head {}
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
impl View for Footer {
    fn render(&self) -> VirtualNode {
        html! {
            <footer>
                <div class="wrapper">
                    <div id="disclaimer">
                        <div>{ "Oak Ridge National Laboratory is managed by UT-Batelle LLC for the US Department of Energy" }</div>
                    </div>
                    <div class="logo-wrapper">
                        <div>
                            <img src="img/logo_ornl.svg" id="logo-ornl"/>
                        </div>
                        <div>
                            <img src="img/logo_doe_os.png" id="logo-doe-os"/>
                        </div>
                        <div id="nss">{ "National Security Sciences" }</div>
                    </div>
                </div>
            </footer>
        }
    }
}
impl View for Head {
    fn render(&self) -> VirtualNode {
        let css = r#"
        @font-face {
            font-family: "myriad";
            src: url("fonts/Myriad\ Pro\ Light.otf") format("opentype");
            font-style: normal;
            font-weight: 300;
        }

        @font-face {
            font-family: "myriad-semi";
            src: url("fonts/Myriad\ Pro\ Semibold.OTF") format("opentype");
            font-style: normal;
            font-weight: 300;
        }

        @font-face {
            font-family: "proxima";
            src: url("fonts/Proxima\ Nova\ Light.otf") format("opentype");
            font-weight: normal;
            font-style: normal;
        }

        @font-face {
            font-family: "proxima-bold";
            src: url("fonts/Proxima\ Nova\ Bold.otf") format("opentype");
            font-weight: bold;
            font-style: normal;
        }

        @font-face {
            font-family: "proxima-semi";
            src: url("fonts/Proxima\ Nova\ Semibold.otf") format("opentype");
            font-weight: bold;
            font-style: normal;
        }

        @font-face {
            font-family: "proxima";
            src: url("fonts/Proxima\ Nova\ Regular\ Italic.otf") format("opentype");
            font-weight: normal;
            font-style: italic;
        }

        @page {
            margin: 0;
            size: letter portrait;
        }

        html,
        body {
            font-family: "myriad";
            margin: 0;
            padding: 0;
        }

        h1,
        h2,
        h3 {
            margin: 0;
            padding: 0;
        }

        header {
            margin: 0;
            padding: 0;
            padding-left: 55px;
            background-image: url("img/banner.jpg");
            background-repeat: no-repeat;
            background-position: bottom center;
            background-size: cover;
            height: 195px;
            position: fixed;
            width: 100%;
            top: 0;
        }

        .main {
            margin-top: 220px;
            position: absolute;
            display: grid;
            grid-template-columns: 500px auto;
            grid-template-rows: auto auto auto;
            grid-gap: 0px;
        }

        main {
            font-family: "myriad";
            font-size: 11pt;
            margin: 0px 0px 0px 50px;
            letter-spacing: 0.1em;
        }

        ul {
            display: block;
            list-style-type: disc;
            list-style-position: outside;
        }

        li {
            display: list-item;
            margin-left: -20px;
        }

        section>h3 {
            color: #16537B;
            font-family: "proxima-bold";
            font-weight: bold;
            font-size: 16pt;
            letter-spacing: 0.1em;
            padding-top: 10px;
            text-transform: uppercase;
        }

        .main>aside {
            grid-column: 2;
            grid-row: 1;
        }

        aside>.graphic {
            background-color: #FFFFFF;
            height: 200px;
            margin: 20px;
            position: relative;
        }

        aside>.qrcode {
            text-align: center;
        }

        aside>.graphic>img {
            width: 100%;
            /* weasyprint */
        }

        aside>.research-focus {
            background-color: #e6e7e8;
            height: 200px;
            letter-spacing: 0.1em;
            margin: 10px 0px 30px 20px;
            padding: 20px 30px 20px 20px;
            position: relative;
        }

        aside>.research-focus>h3 {
            font-size: 13pt;
            font-family: "proxima-bold";
            font-weight: bold;
            letter-spacing: 0.15em;
        }

        aside>div>h3 {
            color: #16537B;
            font-family: "proxima";
            font-size: 11pt;
        }

        footer {
            background-image: url("img/footer.jpg");
            background-position: bottom center;
            background-repeat: no-repeat;
            background-size: cover;
            bottom: 0;
            height: 110px;
            margin: 0;
            padding: 0;
            position: fixed;
            width: 100%;
        }

        .wrapper {
            display: grid;
        }

        header>.wrapper {
            align-items: center;
            grid-gap: 5px;
            grid-template-columns: 70px auto;
            grid-template-rows: auto auto auto;
            margin-top: 45px;
        }

        footer>.wrapper {
            align-items: center;
            color: #FFFFFF;
            grid-gap: 5px;
            grid-template-columns: auto;
            grid-template-rows: auto auto;
            margin-top: 0;
        }

        footer>.wrapper>.logo-wrapper {
            display: grid;
            grid-template-columns: auto auto 50%;
            grid-template-rows: auto;
            text-align: center;
            height: 66px;
            line-height: 66px;
        }

        #contact {
            background-color: #e6e7e8;
            bottom: 110px;
            position: fixed;
            text-align: center;
            width: 100%;
        }

        #contact>div {
            align-items: center;
            justify-content: center;
            display: grid;
            grid-gap: 7px;
            grid-template-columns: auto auto auto 5px auto 5px auto 5px auto;
            grid-template-rows: auto;
            padding: 10px 0;
            margin-left: -50px;
            /* weasyprint */
            white-space: nowrap;
        }

        #contact .email {
            color: #0378c6;
        }

        #contact .label {
            color: #16537B;
            font-family: "proxima-bold";
            font-size: 16pt;
            text-transform: uppercase;
        }

        #contact .label+.spacer {
            margin: 0 9px;
            color: #16537B;
            font-family: "proxima-bold";
            height: 35px;
            width: 3px;
            background-color: #16537B;
            border: 0px solid #16537B;
        }

        #logo {
            grid-column: 1;
            grid-row: 1 / 4;
            width: 60px;
        }

        #logo-doe-os {
            height: 28px;
            padding-top: 15px;
        }

        #logo-ornl {
            height: 32px;
            padding-top: 13px;
        }

        #ornl-header {
            grid-column: 2;
            grid-row: 1;
            color: #FFFFFF;
            font-family: "proxima-semi";
            font-size: 11pt;
            font-weight: 400;
            letter-spacing: 0.35em;
            text-transform: uppercase;
        }

        #title {
            grid-column: 2;
            grid-row: 2;
            color: #FFFFFF;
            font-family: "proxima-bold";
            font-size: 30pt;
            font-weight: bold;
        }

        #subtitle {
            grid-column: 2;
            grid-row: 3;
            color: #FFFFFF;
            font-family: "proxima";
            font-size: 16pt;
            font-style: italic;
            font-weight: normal;
        }

        #disclaimer {
            font-family: "myriad-semi";
            font-size: 10pt;
            grid-column: 1;
            grid-row: 1;
            height: 31px;
            line-height: 31px;
            text-align: center;
        }

        #nss {
            font-family: "proxima-semi";
            font-size: 12pt;
            letter-spacing: 0.3em;
            text-transform: uppercase;
        }
        "#;
        html! {
            <head>
                <meta charset="utf-8">
                <title>{ "Research Activity" }</title>
                <link href="pdf.css" rel="stylesheet">
                <meta name="description" content="Research Activity Data">
                <style>{ css }</style>
            </head>
        }
    }
}
// impl View for ResearchActivity {
//     fn render(&self) -> VirtualNode {

//     }
// }

pub fn project(data: ResearchActivity) -> VirtualNode {
    let ResearchActivity {
        title, subtitle, ..
    } = data;
    let head = Head {};
    let footer = Footer {};
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
                        <header>
                            <div class="wrapper">
                                <img id="logo" src="img/logo_leaf.svg" height="76px"/>
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
