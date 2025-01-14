use crate::schema::{Contact, ProjectSections, Research, ResearchActivity, Sections};
use percy_dom::prelude::*;

pub fn project(data: ResearchActivity) -> VirtualNode {
    let ResearchActivity {
        title, subtitle, ..
    } = data;
    let Contact {
        first,
        last,
        title: role,
        email,
        phone,
        ..
    } = data.contact;
    let head = html! {
        <head>
            <meta charset="utf-8">
            <title>{ "Fact Sheet" }</title>
            <link href="pdf.css" rel="stylesheet">
            <meta name="description" content="Fact Sheet">
        </head>
    };
    let header = html! {
        <header>
            <div class="wrapper">
                <img id="logo" src="img/logo_leaf.svg" height="76px"/>
                <div id="ornl-header">{ "Oak Ridge National Laboratory" }</div>
                <div id="title">{ title }</div>
                <div id="subtitle">{ subtitle }</div>
            </div>
        </header>
    };
    let contact = html! {
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
    };
    let footer = html! {
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
    };
    match data.sections {
        Sections::Highlight(_) => unimplemented!("Highlight template not implemented yet"),
        Sections::Project(sections) => {
            let ProjectSections {
                introduction,
                challenge,
                approach,
                outcomes,
                research,
                ..
            } = sections;
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
                        { header }
                        <div class="main">
                            { main }
                            { aside }
                        </div>
                        { contact }
                        { footer }
                    </body>
                </html>
            }
        }
        Sections::Organization(_) => unimplemented!("Organization template not implemented yet"),
    }
}
