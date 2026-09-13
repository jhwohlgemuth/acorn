// @acorn-exec-v0.2.0
fn evaluate(context) {
    #{ findings: [ #{ rule_id: "demo", severity: "warning", message: "example", locator: context["path"] } ] }
}
