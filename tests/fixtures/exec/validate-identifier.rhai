// @acorn-exec-v0.2.0
fn evaluate(context) {
    if context.input.meta.id == "" {
        #{ findings: [#{
            rule_id: "missing-project-id",
            severity: "error",
            message: "Project metadata requires a stable identifier",
            locator: "meta.id"
        }] }
    } else {
        #{ findings: [] }
    }
}
