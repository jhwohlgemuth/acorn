//! CLI embedded assets for shared logbook report HTML composition.
use crate::template::{generate_font_faces, DataUri, Style, ORNL_COLOR_LOGO, ORNL_LEAF_LOGO, ORNL_WHITE_LOGO};
use acorn::io::{
    export::updates::html::{self as updates_html, LogbookUpdatesHtml as SharedLogbookUpdatesHtml, ReportHtmlAssets},
    ApiResult,
};
use acorn_schema::research_activity::LogbookUpdates;

pub(crate) trait LogbookUpdatesHtml {
    fn to_html(&self, full_timestamp: bool) -> ApiResult<String>;
    fn to_title_html(&self, full_timestamp: bool) -> ApiResult<String>;
}
impl LogbookUpdatesHtml for LogbookUpdates {
    fn to_html(&self, full_timestamp: bool) -> ApiResult<String> {
        assets().map(|assets| SharedLogbookUpdatesHtml::to_html(self, full_timestamp, &assets))
    }
    fn to_title_html(&self, full_timestamp: bool) -> ApiResult<String> {
        assets().map(|assets| SharedLogbookUpdatesHtml::to_title_html(self, full_timestamp, &assets))
    }
}
pub(crate) fn assets() -> ApiResult<ReportHtmlAssets> {
    DataUri::from_asset(ORNL_COLOR_LOGO)
        .and_then(|color_logo| DataUri::from_asset(ORNL_WHITE_LOGO).map(|white_logo| (color_logo, white_logo)))
        .and_then(|(color_logo, white_logo)| DataUri::from_asset(ORNL_LEAF_LOGO).map(|leaf_logo| (color_logo, white_logo, leaf_logo)))
        .and_then(|(color_logo, white_logo, leaf_logo)| {
            DataUri::from_asset("report/title-banner.png").map(|title_banner| (color_logo, white_logo, leaf_logo, title_banner))
        })
        .and_then(|(color_logo, white_logo, leaf_logo, title_banner)| {
            generate_font_faces().map(|font_faces| (color_logo, white_logo, leaf_logo, title_banner, font_faces))
        })
        .and_then(|(color_logo, white_logo, leaf_logo, title_banner, font_faces)| {
            Style::to_string("updates.css".to_string()).map(|style| {
                ReportHtmlAssets::init()
                    .color_logo(color_logo)
                    .white_logo(white_logo)
                    .leaf_logo(leaf_logo)
                    .title_banner(title_banner)
                    .font_faces(font_faces)
                    .style(style)
                    .build()
            })
        })
}
pub(crate) fn combined_html(title: &LogbookUpdates, projects: &[LogbookUpdates], full_timestamp: bool) -> ApiResult<String> {
    assets().map(|assets| updates_html::combine(title, projects, full_timestamp, &assets))
}
pub(crate) fn legend_html() -> ApiResult<String> {
    assets().map(|assets| updates_html::legend_html(&assets))
}
