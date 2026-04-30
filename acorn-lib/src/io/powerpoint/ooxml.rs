//! ## OOXML data structures
//!
//! Data structures for modeling [`OOXML`].
//!
//! [`OOXML`]: https://en.wikipedia.org/wiki/Office_Open_XML
use bon::Builder;
use core::fmt;
use core::iter::once;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

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
impl XmlRels for Vec<Relationship> {
    fn largest_revision_identifier(&self) -> Option<u32> {
        self.revision_identifiers().iter().max().cloned()
    }
    fn revision_identifiers(&self) -> Vec<u32> {
        self.clone()
            .iter()
            .map(|x| x.id.clone().trim_start_matches("rId").to_string().parse::<u32>().unwrap())
            .collect::<Vec<u32>>()
    }
}
