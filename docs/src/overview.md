<style>
    .ayu img.banner.light,
    .coal img.banner.light,
    .navy img.banner.light {
        display: none;
    }
    .light img.banner.dark,
    .rust img.banner.dark {
        display: none;
    }
</style>
<div align="center">
    <p>
        <img class="banner light" src="_assets/banner_light_mode.png" alt="ACORN Banner" height="150" />
        <img class="banner dark" src="_assets/banner_dark_mode.png" alt="ACORN Banner" height="150" />
    </p>
</div>

> [!WARNING]  
> This documentation is a work in progress. Some sections may be incomplete or subject to change.

🌱 ACORN[^acronym] provides an ontology for research activity data (RAD) and enables adding linked data context and transforming RAD into a knowledge graph that is amenable to automated reasoning and artifact generation (e.g. PDFs, PPTX, etc.)

ACORN is a command line multi-tool that employs automated processes for informing and enforcing defined content schemas. With these content schemas, ACORN builds communication assets such as PDFs, presentation files, and web pages. It also lays the foundation for deep data insights about ORNL's — and any institution's — corpus of research. Built using the memory-safe Rust programming language, ACORN can be used on any Windows, Mac, or Linux machine

[^acronym]: ACORN stands for "Accessible Content Optimization for Research Needs

<br/>

![ACORN Input/Output](_assets/acorn_io.png)