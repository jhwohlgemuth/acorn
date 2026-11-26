//! Images associated with loaded research activities.
use crate::session::{self, Workspace};
use acorn_core::util::MimeType;
use data_encoding::BASE64;
use dioxus::prelude::*;
use serde_json::Value;

/// One carousel image or an explicit missing-image placeholder.
#[derive(Clone, PartialEq)]
pub struct Slide {
    pub(crate) src: String,
    pub(crate) caption: String,
    pub(crate) alt: String,
}
impl Slide {
    /// Caption displayed with the slide image.
    pub fn caption(&self) -> &str {
        &self.caption
    }
    /// URL or data URL used to display the slide image.
    pub fn src(&self) -> &str {
        &self.src
    }
    /// Build the single fallback slide or a slide for an explicit missing reference.
    pub fn placeholder(caption: &str) -> Self {
        Self {
            src: asset!("/assets/project-placeholder.svg").to_string(),
            alt: format!("Image unavailable: {caption}"),
            caption: caption.to_owned(),
        }
    }
}
impl Workspace {
    /// Data URL for a loaded raster or SVG image.
    pub fn image_file(&self, path: &str) -> Option<String> {
        self.files.iter().find(|file| file.path == path).and_then(|file| {
            let mime = MimeType::from(path);
            if mime.is_raster_image() {
                Some(file.content.clone())
            } else if mime.is_svg() {
                Some(format!("data:{mime};base64,{}", BASE64.encode(file.content.as_bytes())))
            } else {
                None
            }
        })
    }
    /// Carousel slides for a validated activity, including one fallback if no images exist.
    pub fn slides(&self, record_path: &str, value: &Value) -> Vec<Slide> {
        let images = self.activity_slides(record_path, value);
        if images.is_empty() {
            vec![Slide::placeholder("No image available")]
        } else {
            images
        }
    }
    fn activity_slides(&self, record_path: &str, value: &Value) -> Vec<Slide> {
        let folder = record_path.rsplit_once('/').map_or("", |(folder, _)| folder);
        let media = value.pointer("/meta/media").map(session::many).unwrap_or_default();
        let referenced = media
            .into_iter()
            .filter_map(|item| {
                item.get("contentUrl")
                    .and_then(Value::as_str)
                    .filter(|url| is_image_reference(item, url))
                    .map(|url| (url, item))
            })
            .map(|(url, item)| {
                let caption = item.get("caption").and_then(Value::as_str).unwrap_or(url);
                self.media_path(folder, url).and_then(|path| self.image_file(&path)).map_or_else(
                    || Slide::placeholder(caption),
                    |src| Slide {
                        src,
                        alt: caption.to_owned(),
                        caption: caption.to_owned(),
                    },
                )
            })
            .collect::<Vec<_>>();
        if referenced.is_empty() {
            let prefix = if folder.is_empty() { String::new() } else { format!("{folder}/") };
            self.files
                .iter()
                .filter(|file| file.path.strip_prefix(&prefix).is_some_and(|suffix| !suffix.contains('/')))
                .filter_map(|file| {
                    self.image_file(&file.path).map(|src| Slide {
                        src,
                        caption: file.path.rsplit('/').next().unwrap_or(&file.path).to_owned(),
                        alt: format!("Illustration for {record_path}"),
                    })
                })
                .collect()
        } else {
            referenced
        }
    }
    fn media_path(&self, folder: &str, url: &str) -> Option<String> {
        let reference = url.split(['?', '#']).next().unwrap_or(url);
        if reference.starts_with("https://") || reference.starts_with("http://") {
            self.files
                .iter()
                .find(|file| reference.ends_with(&format!("/{}", file.path)) && MimeType::from(file.path.as_str()).is_image())
                .map(|file| file.path.clone())
        } else {
            let path = if folder.is_empty() {
                reference.to_owned()
            } else {
                format!("{folder}/{reference}")
            };
            (!reference.starts_with('/') && !reference.contains("..") && session::valid_path(&path)).then_some(path)
        }
    }
    /// Aggregate project images, retaining explicit missing references and one empty fallback.
    pub fn project_slides(&self, paths: &[String]) -> Vec<Slide> {
        let images = paths
            .iter()
            .flat_map(|path| {
                self.validations
                    .get(path)
                    .and_then(|result| result.get("value"))
                    .map(|value| self.activity_slides(path, value))
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>();
        if images.is_empty() {
            vec![Slide::placeholder("No image available")]
        } else {
            images
        }
    }
}
/// Accessible image carousel for a project or activity.
#[component]
pub fn ImageCarousel(slides: Vec<Slide>, label: String, compact: bool) -> Element {
    let count = slides.len();
    rsx! {
        div { class: if compact { "image-carousel compact" } else { "image-carousel" }, "data-carousel": "true", role: "region", "aria-roledescription": "carousel", "aria-label": "Images for {label}",
            div { class: "carousel-track",
                for (index, slide) in slides.iter().enumerate() {
                    figure { class: "carousel-slide", "data-slide-index": index,
                        img { class: "catalog-image", src: "{slide.src}", alt: "{slide.alt}", loading: "lazy" }
                        if !compact { figcaption { "{slide.caption}" } }
                    }
                }
            }
            if count > 1 { div { class: "carousel-controls",
                button { r#type: "button", "data-carousel-action": "previous", "aria-label": "Previous image for {label}", disabled: true,
                    svg { view_box: "0 0 24 24", "aria-hidden": "true", path { d: "m15 18-6-6 6-6" } }
                }
                span { class: "carousel-count", role: "status", "aria-live": "polite", "1 / {count}" }
                button { r#type: "button", "data-carousel-action": "next", "aria-label": "Next image for {label}",
                    svg { view_box: "0 0 24 24", "aria-hidden": "true", path { d: "m9 18 6-6-6-6" } }
                }
            } }
        }
    }
}
fn is_image_reference(item: &Value, url: &str) -> bool {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    item.get("caption").and_then(Value::as_str).is_some() && MimeType::from(path).is_image()
}
