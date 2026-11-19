use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub(crate) struct ChartSegment {
    pub(crate) label: String,
    pub(crate) count: usize,
    pub(crate) color: String,
}
#[component]
pub(crate) fn DashboardTile(class: &'static str, children: Element) -> Element {
    rsx! { div { class: "card dashboard-tile {class}", {children} } }
}
#[component]
pub(crate) fn DonutTile(
    class: &'static str,
    title: &'static str,
    center_label: &'static str,
    empty_message: &'static str,
    segments: Vec<ChartSegment>,
    #[props(default)] description: Option<&'static str>,
) -> Element {
    let total = segments.iter().map(|segment| segment.count).sum::<usize>();
    let gradient = donut_gradient(&segments);
    rsx! { DashboardTile { class,
        h2 { "{title}" }
        if let Some(description) = description { p { class: "small muted", "{description}" } }
        if total == 0 { p { class: "muted", "{empty_message}" } }
        else { div { class: "donut-content",
            div { class: "donut-chart", style: format!("background:conic-gradient({gradient})"), "aria-hidden": "true",
                div { class: "donut-center", strong { "{total}" } small { "{center_label}" } }
            }
            div { class: "donut-legend", for segment in segments {
                div { class: "donut-legend-row",
                    span { class: "donut-legend-label", span { class: "donut-swatch", style: format!("background:{}", segment.color), "aria-hidden": "true" } "{segment.label}" }
                    strong { "{segment.count}" }
                }
            } }
        } }
    } }
}
#[component]
pub(crate) fn MetricTile(label: &'static str, count: usize) -> Element {
    rsx! { DashboardTile { class: "metric", strong { "{count}" } small { "{label}" } } }
}
fn donut_gradient(segments: &[ChartSegment]) -> String {
    let denominator = segments.iter().map(|segment| segment.count).sum::<usize>().max(1) as f64;
    segments
        .iter()
        .scan(0_usize, |offset, segment| {
            let start = *offset as f64 * 100.0 / denominator;
            *offset = offset.saturating_add(segment.count);
            let end = *offset as f64 * 100.0 / denominator;
            Some(format!("{} {start:.2}% {end:.2}%", segment.color))
        })
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests;
