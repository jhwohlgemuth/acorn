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

<div align="center">
  <div>
    <a href="https://code.ornl.gov/research-enablement/acorn">
        <img alt="gitlab" src="https://img.shields.io/badge/code.ornl.gov-read_the_code-00662C?style=for-the-badge&logo=gitlab&color=%2300662C
" height="20">
    </a>
    <a href="https://crates.io/crates/acorn-cli">
        <img alt="crates.io" src="https://img.shields.io/crates/v/acorn-cli.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">
    </a>
    <a href="https://code.ornl.gov/research-enablement/acorn/-/releases">
        <img alt="Releases" src="https://code.ornl.gov/research-enablement/acorn/-/badges/release.svg?style=flat-square" height="20">
    </a>
  </div>
</div>
<br/>

🌱 ACORN[^acronym] is a research assistant and metascience multi-tool that provides and operationalizes an ontology for research activity data (RAD). ACORN enables adding linked data context and transforming RAD into a knowledge graph that is amenable to automated reasoning and artifact generation (e.g. PDFs, PPTX, etc.)

ACORN is a command line multi-tool that employs automated processes for informing and enforcing defined content schemas. With these content schemas, ACORN builds communication assets such as PDFs, presentation files, and web pages. It also lays the foundation for deep data insights about ORNL's — and any institution's — corpus of research. Built using the memory-safe Rust programming language, ACORN can be used on any Windows, Mac, or Linux machine

[^acronym]: ACORN stands for "Accessible Content Optimization for Research Needs

<br/>

![ACORN Input/Output](_assets/acorn_io.png)