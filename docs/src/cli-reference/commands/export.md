# 📤 Export
> Export research activity data to various formats for analysis and sharing

This command allows you to export research activity data from ACORN into different formats such as PDF, Markdown, YAML, or PPTX. With `export`, you can easily share your research with sponsors, collaborators, or the general public in a variety of contexts.

`export` supports exporting individual research activity indices or entire directories containing multiple indices.

ACORN allows one to maintain research activity data as persistent interconnected [single sources of truth](https://en.wikipedia.org/wiki/Single_source_of_truth), which allows one to easily create a variety of output artifacts while ensuring consistency and accuracy across all selected formats.

```mermaid
graph LR
    data("JSON</br>(index.json)") --> export{Export}
    export --> PDF("PDF</br>(fact sheet)")
    export --> PPTX("PPTX</br>(presentation)")
    export --> more("More to come...</br>(Markdown, YAML, etc.)")
```

> [!TIP]
> You can see the result of the export command in action by visiting the [ORNL Research Activity Index](https://research.ornl.gov), which features a variety of research activity data presented in different formats.

## Example Usage
```sh
# Export research activity data to PDF fact sheet
acorn export path/to/index.json --format pdf

# Create PowerPoint presentations from all research activity data in a directory
acorn export path/to/project/ --format powerpoint --target highlight
```

## PowerPoint Reference Template
You can customize the PowerPoint export by providing a reference template using the `--reference` option. This allows you to define specific styles, layouts, and branding for your presentations.

```sh
acorn export path/to/index.json \
    --format powerpoint \
    --target highlight \
    --reference path/to/reference.pptx
```

The reference template allows you to specify which and where certain values are used using placeholder text in the format `{{ PLACEHOLDER_NAME }}`. During export, ACORN will replace these placeholders with the corresponding data from the associated research activity data. You can find an example PowerPoint reference template used for testing in the [ACORN GitLab repository](https://code.ornl.gov/research-enablement/acorn/-/blob/main/tests/fixtures/data/highlight/reference.pptx?ref_type=heads).

### Available Placeholders

The following placeholders can be used in your PowerPoint reference template:

#### String values
- `caption` - First image caption
- `challenge` - Challenge description
- `citation` - DOI citation
- `email` - Contact email
- `first` - Contact first name
- `focus` - Research focus area
- `last` - Contact last name
- `managers` - Manager names (joined with `"and"`)
- `mission`
- `notes` - Presentation notes (intended to be added PowerPoint speaker notes)
- `partners` - Partner names (joined with `", "`)
- `programs` - Program names (joined with `"and"`)
- `subtitle`
- `title`

#### Lists (bullet points)
- `achievement`
- `areas` - Research areas
- `impact`
- `technical` - Technical approach