# Concepts

<span title="Accessible Content Optimization for Research Needs">ACORN</span> is built around a few core concepts to ensure its effectiveness. This section will introduce these concepts and explain their significance in the context of <span title="Accessible Content Optimization for Research Needs">ACORN</span> and science.

## Motivation
For the last few decades, scientific progress has been driven largely by publishing papers in, ideally peer-reviewed, scientific journals. Publishing affords researchers recognition, career advancement, and funding opportunities. This model can be effective, but it has some serious limitations <span title="Accessible Content Optimization for Research Needs">ACORN</span> aims to address.
1. **Publish or perish**[^publish]: Scientists and researchers often care more about *publishing* than about sharing knowledge. This is not without reason, as publications are a key metric for career advancement and funding. However, this can lead to a focus on quantity over quality and a reluctance to share negative results or data that does not support a hypothesis.
2. **Reproducibility crisis**[^reproducibility]: Many scientific results are difficult or impossible to reproduce, leading to questions about their validity. This is compounded by the fact that many publications do not provide access to the underlying data or code used in the research.
3. **No clear way to demonstrate science**: [Publications, people, and budgets do not tell the whole story](/motivation.html#-how-is-it-novel). Science needs a cross-domain standard to collect and communicate the full context of scientific endeavors, including data, code, methods, and results.

<span title="Accessible Content Optimization for Research Needs">ACORN</span> addresses these issues by codifying the details of a research project and providing an automated framework for sharing and analyzing scientific knowledge. Predicated on the idea of applying "science all the way down", <span title="Accessible Content Optimization for Research Needs">ACORN</span> applies rigorous scientific principles to the management and dissemination of scientific knowledge.

## Research Enablement
ACORN is managed by the Research Enablement Initiative at Oak Ridge National Laboratory (ORNL). REI aims to improve the way scientific research is conducted, shared, and evaluated. It is a team of developers, communication experts, and information scientists passionate about open science, transparency, and improving the researcher experience.

REI provides:
- A cross-domain model of research activity data (<span title="Research Activity Data">RAD</span>)
    - [ACORN](/concepts/acorn.html) - Schema, controlled vocabularies, and ontologies for describing research activities
    - [ASPECT](/concepts/aspect/index.html) - Attribute of <span title="Accessible Content Optimization for Research Needs">ACORN</span> that describes technology associated with a given research activity
- A command-line application written in Rust - [`acorn-cli`](https://crates.io/crates/acorn-cli)
- A Rust crate for working with <span title="Research Activity Data">RAD</span> - [`acorn-lib`](https://docs.rs/acorn-lib/latest/acorn/)
- A Python package for working with <span title="Research Activity Data">RAD</span> - [`acorn-py`](https://pypi.org/project/acorn-py/)
- A catalog of research activity data at <span title="Oak Ridge National Laboratory">ORNL</span> - [research.ornl.gov](https://research.ornl.gov)

### 🪣 Buckets
RAD lives in versioned folder collections called "buckets". Each bucket contains a set of files and media assets that describe research activities. For multiple examples, see [ORNL's buckets](https://code.ornl.gov/research-enablement/buckets).

Buckets can be combined via a flat-file[^flat] [configuration file](/cli-reference/configuration.html) using the ACORN CLI tool. This allows users to aggregate data from multiple sources and generate outputs such as reports based on the combined data.

Buckets are flexible and extensible - *they do not require a cloud provider or expensive infrastructure to use*. They can be stored in any version-controlled repository, such as GitLab, GitHub, or even a local file system.

Buckets are designed to enable federation and scaling while maintaining low-level control over permissions and access. This allows organizations to share data across teams and departments while maintaining control over who can access and modify the data.

> [!TIP]
> See the [research enablement wiki](https://code.ornl.gov/groups/research-enablement/-/wikis/Buckets) for more information on buckets.


[^publish]: D. R. Grimes, C. T. Bauch, and J. P. A. Ioannidis, "Modelling science trustworthiness under publish or perish pressure,"" Royal Society Open Science, vol. 5, no. 1, p. 171511, Jan. 2018, doi: [10.1098/rsos.171511](https://royalsocietypublishing.org/rsos/article/5/1/171511/93707/Modelling-science-trustworthiness-under-publish-or).

[^reproducibility]: M. Baker, "1,500 scientists lift the lid on reproducibility,"" Nature, vol. 533, no. 7604, Art. no. 7604, May 2016, doi: [10.1038/533452a](https://www.nature.com/articles/533452a).

[^flat]: A flat-file is a simple text file that contains data in a structured format, such as [JSON](https://www.json.org/json-en.html) or [YAML](https://yaml.org/). Flat-files are easy to read and write, and can be used to store configuration data for applications.