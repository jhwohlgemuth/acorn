// @acorn-exec-v0.2.0
fn evaluate(context) {
    if context.input.meta.draft {
        #{ findings: [#{
            rule_id: "draft-project",
            severity: "warning",
            message: "Project metadata is still marked as draft",
            locator: "meta.draft"
        }] }
    } else {
        #{ findings: [] }
    }
}
