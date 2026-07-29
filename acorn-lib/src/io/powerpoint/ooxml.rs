//! ## OOXML data structures
//!
//! Data structures for modeling [`OOXML`].
//!
//! [`OOXML`]: https://en.wikipedia.org/wiki/Office_Open_XML
use crate::io::{read_file, ApiResult};
use crate::prelude::PathBuf;
use crate::util::{Label, StringConversion};
use bon::Builder;
use color_eyre::eyre::{eyre, Report};
use core::fmt;
use core::iter::once;
use core::str::from_utf8;
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use tracing::{debug, error};

const XML_DECLARATION: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>";
/// Trait for working with OOXML data structures
pub trait XmlRels {
    /// Get the largest revision identifier
    fn largest_revision_identifier(&self) -> Option<u32> {
        None
    }
    /// Get all revision identifiers
    fn revision_identifiers(&self) -> Vec<u32> {
        vec![]
    }
}
/// OOXML Text Capitalization Type
///
/// See <https://datypic.com/sc/ooxml/t-a_ST_TextCapsType.html>
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Capitalization {
    /// No capitalization
    #[default]
    #[serde(rename = "none")]
    NoCap,
    /// All capitalized
    All,
    /// Small caps
    Small,
}
/// OOXML Text Effect
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Effect {
    /// See <https://datypic.com/sc/ooxml/e-a_blur-1.html>
    #[serde(rename(serialize = "a:blur", deserialize = "blur"))]
    Blur {
        /// Blur radius
        #[serde(rename = "@rad")]
        radius: Option<String>,
        /// Grow bounds
        #[serde(rename = "@grow")]
        grow_bounds: Option<String>,
    },
    /// See <https://datypic.com/sc/ooxml/e-a_glow-1.html>
    #[serde(rename(serialize = "a:glow", deserialize = "glow"))]
    Glow,
    /// See <https://datypic.com/sc/ooxml/e-a_reflection-1.html>
    #[serde(rename(serialize = "a:reflection", deserialize = "reflection"))]
    Reflection,
}
/// OOXML Line fill properties
///
/// See <https://datypic.com/sc/ooxml/g-a_EG_LineFillProperties.html>
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LineFill {
    /// No fill
    #[default]
    #[serde(rename(serialize = "a:noFill", deserialize = "noFill"))]
    NoFill,
    /// Solid fill
    #[serde(rename(serialize = "a:solidFill", deserialize = "solidFill"))]
    Solid,
    /// Gradient fill
    #[serde(rename(serialize = "a:gradFill", deserialize = "gradFill"))]
    Gradient,
    /// Pattern fill
    #[serde(rename(serialize = "a:pattFill", deserialize = "pattFill"))]
    Pattern,
}
/// OOXML Text Strike Type
///
/// See <https://datypic.com/sc/ooxml/t-a_ST_TextStrikeType.html>
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Strikethrough {
    /// No strike
    #[default]
    #[serde(rename = "noStrike")]
    NoStrike,
    /// Single strike
    #[serde(rename = "sngStrike")]
    Single,
    /// Double strike
    #[serde(rename = "dblStrike")]
    Double,
}
/// OOXML Text Underline Type
///
/// See <https://datypic.com/sc/ooxml/g-a_EG_TextUnderlineLine.html>
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TextUnderline {
    /// Underline follows text
    #[serde(rename(serialize = "a:uLnTx", deserialize = "uLnTx"))]
    FollowsText,
    /// Underline stroke
    #[serde(rename(serialize = "a:uLn", deserialize = "uLn"))]
    Stroke,
}
/// OOXML Bullet Character (`a:buChar`)
///
/// See <https://www.datypic.com/sc/ooxml/e-a_buChar-1.html>
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BulletCharacter {
    /// Bullet character
    #[serde(rename = "@char")]
    pub character: String,
}
/// OOXML Bullet Color (`a:buClr`)
///
/// See <https://www.datypic.com/sc/ooxml/e-a_buClr-1.html>
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BulletColor {
    /// RGB color model - hex variant
    ///
    /// See <https://www.datypic.com/sc/ooxml/e-a_srgbClr-1.html>
    #[serde(rename(serialize = "a:srgbClr", deserialize = "srgbClr"))]
    pub color: Color,
}
/// OOXML Bullet Font (`a:buFont`)
///
/// See <https://www.datypic.com/sc/ooxml/e-a_buFont-1.html>
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BulletFont {
    /// Text typeface
    #[serde(rename = "@typeface")]
    pub typeface: String,
    /// Panose setting
    ///
    /// See <https://en.wikipedia.org/wiki/PANOSE>
    #[serde(rename = "@panose")]
    pub panose: String,
    /// Similar font family
    #[serde(rename = "@pitchFamily")]
    pub similar_font_family: String,
    /// Similar character set
    #[serde(rename = "@charset")]
    pub charset: String,
}
/// OOXML RGB Color - Hex variant (`a:srgbClr`)
///
/// See <https://www.datypic.com/sc/ooxml/e-a_srgbClr-1.html>
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Color {
    /// Hex color
    #[serde(rename = "@val")]
    pub value: String,
}
/// OOXML Effect Container (`a:effectLst`)
///
/// See <https://datypic.com/sc/ooxml/e-a_effectLst-1.html>
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectContainer {
    /// Effects (e.g. blur, glow, etc.)
    #[serde(rename = "$value")]
    pub effect: Option<Vec<Effect>>,
}
/// OOXML Complex Script Font (`a:cs`)
///
/// See <https://www.datypic.com/sc/ooxml/e-a_cs-1.html>
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FontComplexScript {
    /// Text typeface
    #[serde(rename = "@typeface")]
    pub typeface: String,
}
/// OOXML East Asian Font (`a:ea`)
///
/// See <https://www.datypic.com/sc/ooxml/e-a_ea-1.html>
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FontEastAsian {
    /// Text typeface
    #[serde(rename = "@typeface")]
    pub typeface: String,
}
/// OOXML Symbol Font (`a:sym`)
///
/// See <https://www.datypic.com/sc/ooxml/e-a_sym-1.html>
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FontSymbol {
    /// Text typeface
    #[serde(rename = "@typeface")]
    pub typeface: String,
}
/// OOXML Line (`a:ln`)
///
/// See <https://datypic.com/sc/ooxml/e-a_ln-5.html>
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Line {
    #[serde(rename = "$value")]
    line_fill: LineFill,
}
/// Struct for parsing OOXML relationships from .rel files
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
#[builder(start_fn = init)]
pub struct Relationships {
    /// List of relationships
    #[builder(default = vec![])]
    pub relationship: Vec<Relationship>,
    /// XML Namespace
    #[builder(default = "http://schemas.openxmlformats.org/package/2006/relationships".to_string())]
    #[serde(rename = "@xmlns")]
    pub namespace: String,
}
/// Relationships describe references from parts to other internal resources in the package or to external resources
///
/// See <https://ooxml.info/docs/9/9.2/>
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
#[builder(start_fn = init)]
pub struct Relationship {
    /// Relationship identifier
    #[serde(rename = "@Id")]
    pub id: String,
    /// Relationship type
    #[builder(default = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide".to_string())]
    #[serde(rename = "@Type")]
    pub relationship_type: String,
    /// Target resource identifier
    #[serde(rename = "@Target")]
    pub target: String,
    /// Target mode
    #[serde(rename = "@TargetMode")]
    pub target_mode: Option<String>,
}
/// Root element of a PowerPoint `presentation.xml` document.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "p:presentation")]
pub struct Presentation {
    /// Slide identifier list.
    #[serde(rename = "p:sldIdLst")]
    pub slide_id_list: Option<SlideIdList>,
}
/// List of PowerPoint slide identifiers.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SlideIdList {
    /// Slide identifiers.
    #[serde(rename = "p:sldId")]
    pub slide_ids: Vec<SlideId>,
}
/// PowerPoint slide identifier entry.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SlideId {
    /// Slide identifier.
    #[serde(rename = "@id")]
    pub id: u32,
    /// Relationship identifier.
    #[serde(rename = "@r:id")]
    pub relationship_id: String,
}
/// OOXML Text Character Properties (`a:rPr`)
///
/// See <https://www.datypic.com/sc/ooxml/e-a_rPr-2.html>
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[builder(start_fn = init)]
pub struct TextCharacterProperties {
    /// Baseline
    #[serde(rename = "@baseline")]
    pub baseline: Option<String>,
    /// Bold
    #[serde(rename = "@b")]
    pub bold: Option<String>,
    /// Text capitalization type
    #[serde(rename = "@cap")]
    pub capitalization: Option<Capitalization>,
    /// Dirty
    #[builder(default = "0".to_string())]
    #[serde(rename = "@dirty")]
    pub dirty: String,
    /// Italic
    #[serde(rename = "@i")]
    pub italic: Option<String>,
    /// Kerning
    #[builder(default = "0".to_string())]
    #[serde(rename = "@kern")]
    pub kerning: String,
    /// Kumimoji
    #[serde(rename = "@kumimoji")]
    pub kumimoji: Option<String>,
    /// Language identifier
    #[serde(rename = "@lang")]
    pub language: Option<String>,
    /// No proofing
    #[serde(rename = "@noProof")]
    pub no_proofing: Option<String>,
    /// Normalized heights
    #[serde(rename = "@normalizeH")]
    pub normalize_heights: Option<String>,
    /// Spacing
    #[serde(rename = "@spc")]
    pub spacing: Option<String>,
    /// Font size
    #[serde(rename = "@sz")]
    pub size: Option<String>,
    /// Strikethrough
    #[serde(rename = "@strike")]
    pub strikethrough: Option<Strikethrough>,
    /// Underline
    #[serde(rename = "@u")]
    pub underline: Option<String>,
    /// OOXML Line (`a:ln`)
    ///
    /// See <https://www.datypic.com/sc/ooxml/e-a_ln-5.html>
    #[serde(rename(serialize = "a:ln", deserialize = "ln"))]
    pub line: Option<Line>,
    /// Effect list
    #[serde(rename(serialize = "a:effectlst", deserialize = "effectLst"))]
    pub effect_list: Option<EffectContainer>,
    /// Underline follows text
    #[serde(rename(serialize = "a:uLnTx", deserialize = "uLnTx"))]
    pub underline_follows_text: Option<UnderlineFollowsText>,
    /// Underline stroke
    #[serde(rename(serialize = "a:uLn", deserialize = "uLn"))]
    pub underline_stroke: Option<UnderlineStroke>,
    /// Underline fill properties follow text
    #[serde(rename(serialize = "a:uFillTx", deserialize = "uFillTx"))]
    pub underline_fill_properties_follow_text: Option<UnderlineFillPropertiesFollowText>,
    /// Underline fill
    #[serde(rename(serialize = "a:uFill", deserialize = "uFill"))]
    pub underline_fill: Option<UnderlineFill>,
    /// Complext script font
    #[serde(rename(serialize = "a:cs", deserialize = "cs"))]
    pub font_complex_script: Option<FontComplexScript>,
    /// East Asian font
    #[serde(rename(serialize = "a:ea", deserialize = "ea"))]
    pub font_east_asian: Option<FontEastAsian>,
    /// Symbol font
    #[serde(rename(serialize = "a:sym", deserialize = "sym"))]
    pub font_symbol: Option<FontSymbol>,
}
/// OOXML Text Paragraph (`a:p`)
///
/// See <https://datypic.com/sc/ooxml/e-a_p-1.html>
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[builder(start_fn = init)]
#[serde(rename = "a:p")]
pub struct TextParagraph {
    /// Text paragraph properties
    #[builder(default = Vec::new())]
    #[serde(rename(serialize = "a:pPr", deserialize = "pPr"))]
    pub text_paragraph_properties: Vec<TextParagraphProperties>,
    /// Text runs
    #[builder(default = Vec::new())]
    #[serde(rename(serialize = "a:r", deserialize = "r"))]
    pub text_run: Vec<TextRun>,
    /// OOXML End Paragraph Run Properties (`a:endParaRPr`)
    ///
    /// See <https://datypic.com/sc/ooxml/e-a_endParaRPr-1.html>
    #[serde(rename(serialize = "a:endParaRPr", deserialize = "endParaRPr"))]
    pub end_paragraph_run_properties: Option<TextCharacterProperties>,
}
/// OOXML Text Paragraph Properties (`a:pPr`)
///
/// See <https://datypic.com/sc/ooxml/e-a_pPr-1.html>
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[builder(start_fn = init)]
pub struct TextParagraphProperties {
    /// Indent
    #[serde(rename = "@indent")]
    #[builder(default = "0".to_string())]
    pub indent: String,
    /// Left margin
    #[serde(rename = "@marL")]
    #[builder(default = "0".to_string())]
    pub margin_left: String,
    /// Bullet color
    #[serde(rename(serialize = "a:buClr", deserialize = "buClr"))]
    pub bullet_color: Option<BulletColor>,
    /// Bullet font
    #[serde(rename(serialize = "a:buFont", deserialize = "buFont"))]
    pub bullet_font: Option<BulletFont>,
    /// Bullet character
    #[serde(rename(serialize = "a:buChar", deserialize = "buChar"))]
    pub bullet_character: Option<BulletCharacter>,
    /// Default text run properties
    #[serde(rename(serialize = "a:defRPr", deserialize = "defRPr"))]
    pub default_text_run_properties: Option<TextRunPropertiesDefault>,
}
/// OOXML Text Run (`a:r`)
///
/// See <https://datypic.com/sc/ooxml/e-a_r-1.html>
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[builder(start_fn = init)]
pub struct TextRun {
    /// Text run properties
    #[builder(default = Vec::new())]
    #[serde(rename(serialize = "a:rPr", deserialize = "rPr"))]
    pub text_run_properties: Vec<TextCharacterProperties>,
    /// Text
    #[builder(default = TextString::init().build())]
    #[serde(rename(serialize = "a:t", deserialize = "t"))]
    pub text: TextString,
}
/// OOXML Default Text Run Properties (`a:defRpr`)
///
/// See <https://www.datypic.com/sc/ooxml/e-a_defRPr-1.html>
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TextRunPropertiesDefault {}
/// OOXML Text String (`a:t`)
///
/// See <https://datypic.com/sc/ooxml/e-a_t-1.html>
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[builder(start_fn = init)]
pub struct TextString {
    /// Text value
    #[builder(default = "".to_string())]
    #[serde(rename = "$text")]
    pub value: String,
}
/// OOXML Underline Fill (`a:uFill`)
///
/// See <https://www.datypic.com/sc/ooxml/e-a_uFill-1.html>
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UnderlineFill {}
/// OOXML Underline Fill Properties Follow Text (`a:uFillTx`)
///
/// See <https://www.datypic.com/sc/ooxml/e-a_uFillTx-1.html>
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UnderlineFillPropertiesFollowText {}
/// OOXML Underline Follows Text (`a:uLnTx`)
///
/// See <https://datypic.com/sc/ooxml/e-a_uLnTx-1.html>
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UnderlineFollowsText {}
/// OOXML Underline Stroke (`a:uLn`)
///
/// See <https://www.datypic.com/sc/ooxml/e-a_uLn-1.html>
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UnderlineStroke {}
impl XmlRels for Vec<Relationship> {
    fn largest_revision_identifier(&self) -> Option<u32> {
        self.revision_identifiers().iter().max().cloned()
    }
    fn revision_identifiers(&self) -> Vec<u32> {
        self.clone()
            .iter()
            .filter_map(|x| x.id.clone().trim_start_matches("rId").to_string().parse::<u32>().ok())
            .collect::<Vec<u32>>()
    }
}
impl Default for Relationships {
    fn default() -> Self {
        Self::init().build()
    }
}
impl fmt::Display for Relationships {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.to_string() {
            | Ok(xml) => write!(f, "{}", xml),
            | Err(e) => write!(f, "Error serializing Relationships: {}", e),
        }
    }
}
impl Relationships {
    /// Add a relationship
    pub fn add_relationship(&self, value: Relationship) -> Relationships {
        let Relationships { relationship, .. } = self;
        let updated = relationship.clone().into_iter().chain(once(value)).collect::<Vec<_>>();
        Relationships::init().relationship(updated).build()
    }
    /// Get largest revision identifier among relationships
    pub fn largest_revision_identifier(&self) -> Option<u32> {
        self.relationship.largest_revision_identifier()
    }
    /// Convert to XML string using quick_xml serialization
    pub fn to_string(&self) -> Result<String, quick_xml::de::DeError> {
        let xml = quick_xml::se::to_string(self).map_err(|e| quick_xml::de::DeError::Custom(e.to_string()))?;
        Ok(format!("{}{}", XML_DECLARATION, xml))
    }
}
fn is_png_content_type(element: &BytesStart<'_>) -> ApiResult<bool> {
    element.attributes().try_fold(false, |found, attribute| {
        attribute
            .map(|attribute| found || (attribute.key.as_ref() == b"Extension" && attribute.value.as_ref().eq_ignore_ascii_case(b"png")))
            .map_err(|why| eyre!("Failed to read OOXML content-type attribute — {why}"))
    })
}
fn contains_content_types_root(xml: &str) -> ApiResult<bool> {
    let mut reader = Reader::from_str(xml);
    loop {
        match reader.read_event() {
            | Ok(Event::Start(element)) if element.name().local_name().as_ref() == b"Types" => break Ok(true),
            | Ok(Event::Eof) => break Ok(false),
            | Ok(_) => {}
            | Err(why) => break Err(eyre!("Failed to parse OOXML content-types document — {why}")),
        }
    }
}
fn contains_png_content_type(xml: &str) -> ApiResult<bool> {
    let mut reader = Reader::from_str(xml);
    loop {
        match reader.read_event() {
            | Ok(Event::Start(element) | Event::Empty(element)) if element.name().local_name().as_ref() == b"Default" => {
                match is_png_content_type(&element) {
                    | Ok(true) => break Ok(true),
                    | Ok(false) => {}
                    | Err(why) => break Err(why),
                }
            }
            | Ok(Event::Eof) => break Ok(false),
            | Ok(_) => {}
            | Err(why) => break Err(eyre!("Failed to parse OOXML content-types document — {why}")),
        }
    }
}
fn add_png_content_type(xml: &str) -> ApiResult<String> {
    let mut reader = Reader::from_str(xml);
    let mut writer = Writer::new(Vec::new());
    loop {
        match reader.read_event() {
            | Ok(Event::End(end)) if end.name().local_name().as_ref() == b"Types" => {
                let name = from_utf8(end.name().as_ref())
                    .map_err(|why| eyre!("OOXML content-types root is not UTF-8 — {why}"))
                    .and_then(|name| {
                        name.strip_suffix("Types")
                            .map(|prefix| format!("{prefix}Default"))
                            .ok_or_else(|| eyre!("OOXML content-types document has no Types root"))
                    });
                match name.and_then(|name| {
                    let mut content_type = BytesStart::new(name);
                    content_type.push_attribute(("Extension", "png"));
                    content_type.push_attribute(("ContentType", "image/png"));
                    writer
                        .write_event(Event::Empty(content_type))
                        .and_then(|_| writer.write_event(Event::End(end)))
                        .map_err(|why| eyre!("Failed to add PNG OOXML content type — {why}"))
                        .and_then(|_| {
                            String::from_utf8(writer.into_inner()).map_err(|why| eyre!("OOXML content-types document is not UTF-8 — {why}"))
                        })
                }) {
                    | Ok(value) => break Ok(value),
                    | Err(why) => break Err(why),
                }
            }
            | Ok(Event::Eof) => break Err(eyre!("OOXML content-types document has no Types root")),
            | Ok(event) => match writer.write_event(event.into_owned()) {
                | Ok(_) => {}
                | Err(why) => break Err(eyre!("Failed to write OOXML content-types document — {why}")),
            },
            | Err(why) => break Err(eyre!("Failed to parse OOXML content-types document — {why}")),
        }
    }
}
/// Ensure an OOXML content-types document declares PNG media.
pub fn ensure_png_content_type(xml: &str) -> ApiResult<String> {
    contains_content_types_root(xml).and_then(|has_root| {
        if has_root {
            contains_png_content_type(xml).and_then(|has_png| if has_png { Ok(xml.to_string()) } else { add_png_content_type(xml) })
        } else {
            Err(eyre!("OOXML content-types document has no Types root"))
        }
    })
}
#[derive(Clone, Copy)]
struct PicturePlacement {
    shape_identifier: u32,
    x: u64,
    y: u64,
    width: u64,
    height: u64,
}
fn unsigned_attribute(element: &BytesStart<'_>, name: &[u8]) -> ApiResult<Option<u64>> {
    element
        .attributes()
        .find_map(|attribute| match attribute {
            | Ok(attribute) if attribute.key.as_ref() == name => Some(
                from_utf8(attribute.value.as_ref())
                    .map_err(|why| eyre!("PowerPoint shape attribute is not UTF-8 — {why}"))
                    .and_then(|value| {
                        value
                            .parse::<u64>()
                            .map_err(|why| eyre!("PowerPoint shape attribute is not unsigned — {why}"))
                    }),
            ),
            | Ok(_) => None,
            | Err(why) => Some(Err(eyre!("Failed to read PowerPoint shape attribute — {why}"))),
        })
        .transpose()
}
fn event_unsigned_attribute(events: &[Event<'static>], element_name: &[u8], attribute_name: &[u8]) -> ApiResult<Option<u64>> {
    events
        .iter()
        .find_map(|event| match event {
            | Event::Start(element) | Event::Empty(element) if element.name().as_ref() == element_name => {
                Some(unsigned_attribute(element, attribute_name))
            }
            | _ => None,
        })
        .transpose()
        .map(Option::flatten)
}
impl PicturePlacement {
    fn is_aspect_placeholder(events: &[Event<'static>]) -> bool {
        events
            .iter()
            .filter_map(|event| match event {
                | Event::Text(text) => from_utf8(text.as_ref()).ok(),
                | _ => None,
            })
            .flat_map(str::chars)
            .filter(|character| !character.is_whitespace())
            .collect::<String>()
            .contains("{{aspect}}")
    }
    fn write(self, writer: &mut Writer<Vec<u8>>, relationship_id: &str) -> ApiResult<()> {
        let Self {
            shape_identifier,
            x,
            y,
            width,
            height,
        } = self;
        let picture = format!(
            r#"<p:pic><p:nvPicPr><p:cNvPr id="{shape_identifier}" name="ASPECT rose chart" descr="ASPECT rose chart"/><p:cNvPicPr/><p:nvPr/></p:nvPicPr><p:blipFill><a:blip r:embed="{relationship_id}"/><a:stretch><a:fillRect/></a:stretch></p:blipFill><p:spPr><a:xfrm><a:off x="{x}" y="{y}"/><a:ext cx="{width}" cy="{height}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom><a:noFill/><a:ln><a:noFill/></a:ln></p:spPr></p:pic>"#
        );
        let mut reader = Reader::from_str(&picture);
        loop {
            match reader.read_event() {
                | Ok(Event::Eof) => break Ok(()),
                | Ok(event) => match writer.write_event(event.into_owned()) {
                    | Ok(_) => {}
                    | Err(why) => break Err(eyre!("Failed to write PowerPoint chart picture — {why}")),
                },
                | Err(why) => break Err(eyre!("Failed to parse PowerPoint chart picture — {why}")),
            }
        }
    }
}
impl TryFrom<&[Event<'static>]> for PicturePlacement {
    type Error = Report;
    fn try_from(events: &[Event<'static>]) -> Result<Self, Self::Error> {
        event_unsigned_attribute(events, b"p:cNvPr", b"id")
            .and_then(|shape_identifier| event_unsigned_attribute(events, b"a:off", b"x").map(|x| (shape_identifier, x)))
            .and_then(|(shape_identifier, x)| event_unsigned_attribute(events, b"a:off", b"y").map(|y| (shape_identifier, x, y)))
            .and_then(|(shape_identifier, x, y)| event_unsigned_attribute(events, b"a:ext", b"cx").map(|width| (shape_identifier, x, y, width)))
            .and_then(|(shape_identifier, x, y, width)| {
                event_unsigned_attribute(events, b"a:ext", b"cy").map(|height| (shape_identifier, x, y, width, height))
            })
            .and_then(|(shape_identifier, x, y, width, height)| {
                shape_identifier
                    .and_then(|value| u32::try_from(value).ok())
                    .zip(x)
                    .zip(y)
                    .zip(width)
                    .zip(height)
                    .map(|((((shape_identifier, x), y), width), height)| Self {
                        shape_identifier,
                        x,
                        y,
                        width,
                        height,
                    })
                    .ok_or_else(|| eyre!("PowerPoint ASPECT placeholder has no complete shape bounds"))
            })
    }
}
/// Replace each `{{ aspect }}` slide shape with an image at the same bounds.
pub fn replace_aspect_placeholders_with_picture(xml: &str, relationship_id: &str) -> ApiResult<Option<String>> {
    let mut reader = Reader::from_str(xml);
    let mut writer = Writer::new(Vec::new());
    let mut replacement_count = 0_u32;
    loop {
        match reader.read_event() {
            | Ok(Event::Start(start)) if start.name().as_ref() == b"p:sp" => {
                let mut depth = 1_u32;
                let mut events = vec![Event::Start(start.into_owned())];
                let shape_result = loop {
                    match reader.read_event() {
                        | Ok(Event::Start(event)) => {
                            depth = depth.saturating_add(1);
                            events.push(Event::Start(event.into_owned()));
                        }
                        | Ok(Event::End(event)) => {
                            depth = depth.saturating_sub(1);
                            events.push(Event::End(event.into_owned()));
                            if depth == 0 {
                                break Ok(());
                            }
                        }
                        | Ok(Event::Eof) => break Err(eyre!("PowerPoint slide ended inside a shape")),
                        | Ok(event) => events.push(event.into_owned()),
                        | Err(why) => break Err(eyre!("Failed to parse PowerPoint slide shape — {why}")),
                    }
                };
                match shape_result
                    .and_then(|_| {
                        PicturePlacement::is_aspect_placeholder(&events)
                            .then(|| PicturePlacement::try_from(events.as_slice()))
                            .transpose()
                    })
                    .and_then(|placement| match placement {
                        | Some(placement) => {
                            replacement_count = replacement_count.saturating_add(1);
                            placement.write(&mut writer, relationship_id)
                        }
                        | None => events.into_iter().try_for_each(|event| {
                            writer
                                .write_event(event)
                                .map_err(|why| eyre!("Failed to write PowerPoint slide shape — {why}"))
                        }),
                    }) {
                    | Ok(_) => {}
                    | Err(why) => break Err(why),
                }
            }
            | Ok(Event::Eof) => match String::from_utf8(writer.into_inner()) {
                | Ok(value) if replacement_count > 0 => break Ok(Some(value)),
                | Ok(_) => break Ok(None),
                | Err(why) => break Err(eyre!("PowerPoint slide XML is not UTF-8 — {why}")),
            },
            | Ok(event) => match writer.write_event(event.into_owned()) {
                | Ok(_) => {}
                | Err(why) => break Err(eyre!("Failed to write PowerPoint slide XML — {why}")),
            },
            | Err(why) => break Err(eyre!("Failed to parse PowerPoint slide XML — {why}")),
        }
    }
}
/// Add a slide identifier to a PowerPoint `presentation.xml` document.
pub fn add_slide_to_presentation_xml(xml: &str, slide_identifier: u32, revision_identifier: u32) -> ApiResult<String> {
    let mut reader = Reader::from_str(xml);
    let mut writer = Writer::new(Vec::new());
    let mut found_slide_list = false;
    loop {
        match reader.read_event() {
            | Ok(Event::End(end)) if end.name().as_ref() == b"p:sldIdLst" => {
                found_slide_list = true;
                let mut slide = BytesStart::new("p:sldId");
                let id = slide_identifier.to_string();
                let relationship_id = format!("rId{revision_identifier}");
                slide.push_attribute(("id", id.as_str()));
                slide.push_attribute(("r:id", relationship_id.as_str()));
                match writer.write_event(Event::Empty(slide)).and_then(|_| writer.write_event(Event::End(end))) {
                    | Ok(_) => {}
                    | Err(why) => break Err(eyre!("Failed to write presentation slide identifier — {why}")),
                }
            }
            | Ok(Event::Eof) if found_slide_list => {
                let output = writer.into_inner();
                match String::from_utf8(output) {
                    | Ok(value) => break Ok(value),
                    | Err(why) => break Err(eyre!("Failed to decode presentation.xml as UTF-8 — {why}")),
                }
            }
            | Ok(Event::Eof) => break Err(eyre!("Missing p:sldIdLst in presentation.xml")),
            | Ok(event) => match writer.write_event(event) {
                | Ok(_) => {}
                | Err(why) => break Err(eyre!("Failed to write presentation.xml event — {why}")),
            },
            | Err(why) => break Err(eyre!("Failed to parse presentation.xml at byte {} — {why}", reader.buffer_position())),
        }
    }
}
/// Replace a relationship target, returning an error when the current target is absent.
pub fn update_relationship_target(content: &str, current_target: &str, target: &str) -> ApiResult<String> {
    quick_xml::de::from_str::<Relationships>(content)
        .map_err(|why| eyre!("Failed to parse slide relationships — {why}"))
        .and_then(|rels| match rels.relationship.iter().any(|rel| rel.target == current_target) {
            | true => {
                let updated = rels
                    .relationship
                    .into_iter()
                    .map(|rel| match rel.target == current_target {
                        | true => Relationship::init()
                            .id(rel.id)
                            .relationship_type(rel.relationship_type)
                            .target(target.to_string())
                            .maybe_target_mode(rel.target_mode)
                            .build(),
                        | false => rel,
                    })
                    .collect::<Vec<_>>();
                Relationships::init()
                    .relationship(updated)
                    .build()
                    .to_string()
                    .map_err(|why| eyre!("Failed to serialize slide relationships — {why}"))
            }
            | false => Err(eyre!("Missing relationship target {current_target}")),
        })
}
/// Validate that XML content is well formed.
pub fn validate_xml_well_formed(content: &str) -> bool {
    let mut reader = Reader::from_str(content);
    match content.trim().is_empty() {
        | true => false,
        | false => {
            let mut open_elements = Vec::new();
            loop {
                match reader.read_event() {
                    | Ok(Event::Start(event)) => open_elements.push(event.name().as_ref().to_vec()),
                    | Ok(Event::End(event)) => match open_elements.pop() {
                        | Some(name) if name == event.name().as_ref() => {}
                        | _ => break false,
                    },
                    | Ok(Event::Eof) => break open_elements.is_empty(),
                    | Ok(_) => {}
                    | Err(_) => break false,
                }
            }
        }
    }
}
/// Prettify XML
pub fn prettify_xml(xml: &str) -> String {
    let mut reader = Reader::from_str(xml);
    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);
    loop {
        match reader.read_event() {
            | Ok(Event::Eof) => break,
            | Ok(event) => match writer.write_event(event) {
                | Ok(_) => {}
                | Err(why) => {
                    error!("=> {} Cannot write XML event — {why}", Label::fail());
                    break;
                }
            },
            | Err(why) => {
                error!("=> {} Error at XML position {} — {why}", Label::fail(), reader.buffer_position());
                break;
            }
        }
    }
    let output = writer.into_inner();
    match from_utf8(&output) {
        | Ok(value) => value.to_string(),
        | Err(why) => {
            error!("=> {} Cannot decode prettified XML as UTF-8 — {why}", Label::fail());
            String::new()
        }
    }
}
/// Read OOXML relationships XML file.
pub fn read_xml_rel(path: PathBuf) -> Option<Relationships> {
    match read_file(path.clone()) {
        | Ok(content) => {
            let parsed = quick_xml::de::from_str::<Relationships>(&content);
            debug!("=> {} Relationships = {:#?}", Label::using(), parsed);
            match parsed {
                | Ok(value) => Some(value),
                | Err(why) => {
                    error!(
                        path = path.to_absolute_string(),
                        "=> {} Cannot parse relationships - {why}",
                        Label::fail()
                    );
                    None
                }
            }
        }
        | Err(why) => {
            error!(path = path.to_absolute_string(), "=> {} Cannot read xml.rels file - {why}", Label::fail());
            None
        }
    }
}
