use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// OOXML Text Capitalization Type
///
/// See <https://datypic.com/sc/ooxml/t-a_ST_TextCapsType.html>
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Capitalization {
    #[serde(rename = "none")]
    NoCap,
    All,
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
        #[serde(rename = "@rad")]
        radius: Option<String>,
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
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LineFill {
    #[serde(rename(serialize = "a:noFill", deserialize = "noFill"))]
    NoFill,
    #[serde(rename(serialize = "a:solidFill", deserialize = "solidFill"))]
    Solid,
    #[serde(rename(serialize = "a:gradFill", deserialize = "gradFill"))]
    Gradient,
    #[serde(rename(serialize = "a:pattFill", deserialize = "pattFill"))]
    Pattern,
}
/// OOXML Text Strike Type
///
/// See <https://datypic.com/sc/ooxml/t-a_ST_TextStrikeType.html>
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Strikethrough {
    #[serde(rename = "noStrike")]
    NoStrike,
    #[serde(rename = "sngStrike")]
    Single,
    #[serde(rename = "dblStrike")]
    Double,
}
/// OOXML Text Underline Type
///
/// See <https://datypic.com/sc/ooxml/g-a_EG_TextUnderlineLine.html>
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TextUnderline {
    #[serde(rename(serialize = "a:uLnTx", deserialize = "uLnTx"))]
    FollowsText,
    #[serde(rename(serialize = "a:uLn", deserialize = "uLn"))]
    Stroke,
}
/// OOXML Bullet Character (`a:buChar`)
///
/// See <>
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BulletCharacter {
    #[serde(rename = "@char")]
    pub character: String,
}
/// OOXML Bullet Color (`a:buClr`)
///
/// See <>
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BulletColor {}
/// OOXML Bullet Font (`a:buFont`)
///
/// See <>
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BulletFont {
    #[serde(rename = "@typeface")]
    pub typeface: String,
    #[serde(rename = "@panose")]
    pub panose: String,
    #[serde(rename = "@pitchFamily")]
    pub similar_font_family: String,
    #[serde(rename = "@charset")]
    pub charset: String,
}
/// OOXML Effect Container (`a:effectLst`)
///
/// See <https://datypic.com/sc/ooxml/e-a_effectLst-1.html>
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectContainer {
    #[serde(rename = "$value")]
    pub effect: Option<Vec<Effect>>,
}
/// OOXML Complex Script Font (`a:cs`)
///
/// See <https://www.datypic.com/sc/ooxml/e-a_cs-1.html>
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FontComplexScript {
    #[serde(rename = "@typeface")]
    pub typeface: String,
}
/// OOXML East Asian Font (`a:ea`)
///
/// See <https://www.datypic.com/sc/ooxml/e-a_ea-1.html>
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FontEastAsian {
    #[serde(rename = "@typeface")]
    pub typeface: String,
}
/// OOXML Symbol Font (`a:sym`)
///
/// See <https://www.datypic.com/sc/ooxml/e-a_sym-1.html>
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FontSymbol {
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
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Relationships {
    pub relationship: Vec<Relationship>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Relationship {
    #[serde(rename = "@Id")]
    pub id: String,
    #[serde(rename = "@Type")]
    pub relationship_type: String,
    #[serde(rename = "@Target")]
    pub target: String,
    #[serde(rename = "@TargetMode")]
    pub target_mode: Option<String>,
}
/// OOXML Text Character Properties (`a:rPr`)
///
/// See <https://www.datypic.com/sc/ooxml/e-a_rPr-2.html>
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TextCharacterProperties {
    #[serde(rename = "@baseline")]
    pub baseline: String,
    #[serde(rename = "@b")]
    pub bold: String,
    #[serde(rename = "@cap")]
    pub capitalization: Capitalization,
    #[serde(rename = "@dirty")]
    pub dirty: String,
    #[serde(rename = "@i")]
    pub italic: String,
    #[serde(rename = "@kern")]
    pub kerning: String,
    #[serde(rename = "@kumimoji")]
    pub kumimoji: String,
    #[serde(rename = "@lang")]
    pub language: Option<String>,
    #[serde(rename = "@noProof")]
    pub no_proofing: String,
    #[serde(rename = "@normalizeH")]
    pub normalize_heights: String,
    #[serde(rename = "@spc")]
    pub spacing: String,
    #[serde(rename = "@sz")]
    pub size: String,
    #[serde(rename = "@strike")]
    pub strikethrough: Strikethrough,
    #[serde(rename = "@u")]
    pub underline: String,
    #[serde(rename(serialize = "a:ln", deserialize = "ln"))]
    pub line: Line,
    #[serde(rename(serialize = "a:effectlst", deserialize = "effectLst"))]
    pub effect_list: EffectContainer,
    #[serde(rename(serialize = "a:uLnTx", deserialize = "uLnTx"))]
    pub underline_follows_text: Option<UnderlineFollowsText>,
    #[serde(rename(serialize = "a:uLn", deserialize = "uLn"))]
    pub underline_stroke: Option<UnderlineStroke>,
    #[serde(rename(serialize = "a:uFillTx", deserialize = "uFillTx"))]
    pub underline_fill_properties_follow_text: Option<UnderlineFillPropertiesFollowText>,
    #[serde(rename(serialize = "a:uFill", deserialize = "uFill"))]
    pub underline_fill: Option<UnderlineFill>,
    #[serde(rename(serialize = "a:cs", deserialize = "cs"))]
    pub font_complex_script: Option<FontComplexScript>,
    #[serde(rename(serialize = "a:ea", deserialize = "ea"))]
    pub font_east_asian: Option<FontEastAsian>,
    #[serde(rename(serialize = "a:sym", deserialize = "sym"))]
    pub font_symbol: Option<FontSymbol>,
}
/// OOXML Text Paragraph (`a:p`)
///
/// See <https://datypic.com/sc/ooxml/e-a_p-1.html>
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "a:p")]
pub struct TextParagraph {
    #[serde(rename(serialize = "a:pPr", deserialize = "pPr"))]
    pub text_paragraph_properties: Vec<TextParagraphProperties>,
    #[serde(rename(serialize = "a:r", deserialize = "r"))]
    pub text_run: Vec<TextRun>,
    /// OOXML End Paragraph Run Properties (`a:endParaRPr`)
    ///
    /// See <https://datypic.com/sc/ooxml/e-a_endParaRPr-1.html>
    #[serde(rename(serialize = "a:endParaRPr", deserialize = "endParaRPr"))]
    end_paragraph_run_properties: TextCharacterProperties,
}
/// OOXML Text Paragraph Properties (`a:pPr`)
///
/// See <https://datypic.com/sc/ooxml/e-a_pPr-1.html>
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TextParagraphProperties {
    #[serde(rename = "@indent")]
    pub indent: String,
    #[serde(rename = "@marL")]
    pub margin_left: String,
    #[serde(rename(serialize = "a:buClr", deserialize = "buClr"))]
    pub bullet_color: Option<BulletColor>,
    #[serde(rename(serialize = "a:buFont", deserialize = "buFont"))]
    pub bullet_font: Option<BulletFont>,
    #[serde(rename(serialize = "a:buChar", deserialize = "buChar"))]
    pub bullet_character: Option<BulletCharacter>,
    #[serde(rename(serialize = "a:defRPr", deserialize = "defRPr"))]
    pub default_text_run_properties: Option<TextRunPropertiesDefault>,
}
/// OOXML Text Run (`a:r`)
///
/// See <https://datypic.com/sc/ooxml/e-a_r-1.html>
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TextRun {
    #[serde(rename(serialize = "a:rPr", deserialize = "rPr"))]
    pub text_run_properties: Vec<TextCharacterProperties>,
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
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TextString {
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
