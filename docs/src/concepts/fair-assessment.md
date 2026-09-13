# FAIR evidence assessment

ACORN's `fair` check category describes how strongly the available metadata supports the [FAIR principles](https://www.go-fair.org/fair-principles/). It evaluates the 41 indicators in the [RDA FAIR Data Maturity Model](https://doi.org/10.15497/RDA00050). The RDA recommendation leaves assessment methods to implementers, so ACORN reports its observed evidence instead of inventing a maturity level or compliance grade.

## Evidence states

Every indicator has one of four states:

- **Supported**: available evidence supports the indicator. This is informational and does not fail a check.
- **Unsupported**: available evidence explicitly contradicts the indicator, such as a failed resolution check. This is a warning and fails the check.
- **Indeterminate**: metadata is missing or ACORN cannot prove the indicator. This is a non-failing suggestion.
- **Not applicable**: the indicator does not apply to the assessed resource. This is informational and does not fail the check.

Missing evidence does not mean that data is not FAIR. Likewise, FAIR does not mean open: authenticated or controlled access can support the Accessible principle when it is clearly described.

## Supported metadata

ACORN performs native FAIR assessment for Research Activity Data (RAD), DataCite, DCAT, Huwise, InvenioRDM, and RAiD metadata. Other input standards receive a single suggestion explaining that native assessment is not supported.

For RAD, each item in `meta.outputs` is assessed as a separate resource. Activity-level metadata supplements each output. A command that assesses several resources prints each resource profile and a pooled profile; pooling adds indicator counts without weighting RDA priorities.

Existing link-check results are reused as resolution evidence. ACORN does not make a second request for FAIR assessment. When link checks are disabled, skipped, or offline, resolution indicators remain indeterminate unless other evidence establishes their state.

## Natural-language evidence

Some evidence appears in prose rather than structured fields. Rules for access-on-request language, license wording, and unsupported compliance claims belong in ACORN's external `Science` Vale package because they require sentence-level context. Those rules should remain conservative: controlled access can be FAIR, and a claim that data is FAIR is not itself evidence that every indicator is supported. ACORN's bundled package reference should only move to a release after those rules are published and verified.

## Reading output

The detailed console view uses the same located diagnostic format as other check categories. Its heading contains the RDA indicator ID, principle, and priority. The message describes the evidence or the action needed to strengthen support.

Terse output keeps ACORN's three columns: location or resource, severity, and remediation. Machine-readable details follow the message as `rule`, `state`, and `priority` values. Profile tables are intentionally omitted in terse mode.

CSV output adds one stable `fair` column containing compact rule, principle-indicator, evidence-state, and priority metadata. The column is empty for checks outside the FAIR category. Detailed console output adds a separate per-resource FAIR profile table after the issue summary.

Use `--all` to include supported, indeterminate, and informational indicator results. Use `--skip fair` to omit the category.
