//! ## Data model for patent data
//!
use crate::util::constants::{RE_PATENT, RE_PATENT_TEXT};
use crate::util::regex_capture_lookup;
use derive_more::Display;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Country code for identifying country authority
#[derive(Clone, Debug, Default, Display, Deserialize, Serialize, JsonSchema)]
pub enum CountryCode {
    /// United States
    #[default]
    #[display("US")]
    US,
    /// Chinese
    #[display("CN")]
    CN,
    /// German
    #[display("DE")]
    DE,
    /// European
    #[display("EP")]
    EP,
    /// Japanese
    #[display("JP")]
    JP,
    /// Korean
    #[display("KR")]
    KR,
}
/// Patent and Patent-application document kind codes (WIPO ST.16 style).
///
/// This enum focuses on currently used codes for USPTO patent-related documents (utility, design, plant, reissue, reexamination, etc.).
///
/// ### Notes
/// - Codes follow WIPO ST.16 and USPTO practice (e.g., "US 7,654,321 B2")
/// - Trademark-only codes (e.g., X3, X4, X5, X8) are not included
/// - Some codes are legacy but still appear on older documents (e.g., H, H1)
#[derive(Clone, Copy, Debug, Deserialize, Display, Eq, Hash, PartialEq, Serialize)]
pub enum KindCode {
    /// Utility patent application, first publication (pre‑grant)
    #[display("A1")]
    #[serde(rename = "A1")]
    A1,
    /// Second or subsequent publication of a utility patent application
    #[display("A2")]
    #[serde(rename = "A2")]
    A2,
    /// Utility patent application, corrected publication
    #[display("A9")]
    #[serde(rename = "A9")]
    A9,
    /// Utility patent grant, no pre‑grant publication (post‑Jan 2, 2001 use)
    #[display("B1")]
    #[serde(rename = "B1")]
    B1,
    /// Utility patent grant, with pre‑grant publication
    #[display("B2")]
    #[serde(rename = "B2")]
    B2,
    /// Identifies a reexamination certificate issued after the first reexamination proceeding for a granted US patent (current C‑series replacement for older B1/B2/B3)
    #[display("C1")]
    #[serde(rename = "C1")]
    C1,
    /// Identifies a reexamination certificate issued after a second reexamination proceeding on the same patent (i.e., the patent has been reexamined twice)
    #[display("C2")]
    #[serde(rename = "C2")]
    C2,
    /// Identifies a reexamination certificate issued after a third reexamination proceeding on that patent
    #[display("C3")]
    #[serde(rename = "C3")]
    C3,
    /// Reissue patent (normalized to two‑position form)
    #[display("E1")]
    #[serde(rename = "E1", alias = "E")]
    E1,
    /// Design patent grant (normalized to two‑position form)
    #[display("S1")]
    #[serde(rename = "S1", alias = "S")]
    S1,
    /// Plant application publication, first publication
    #[display("P1")]
    #[serde(rename = "P1")]
    P1,
    /// Plant patent grant, no pre‑grant publication (post‑Jan 2, 2001)
    #[display("P2")]
    #[serde(rename = "P2")]
    P2,
    /// Plant patent grant, with pre‑grant publication (post‑Jan 2, 2001)
    #[display("P3")]
    #[serde(rename = "P3")]
    P3,
    /// Second or subsequent publication of a plant patent application
    #[display("P4")]
    #[serde(rename = "P4")]
    P4,
    /// Corrected publication of a plant patent application
    #[display("P9")]
    #[serde(rename = "P9")]
    P9,
    /// Statutory invention registration (SIR)
    /// ### Note
    /// > SIR program was ended in 2013, so H/H1 appear only on legacy documents.
    #[display("H1")]
    #[serde(rename = "H1", alias = "H")]
    H1,
    /// Unknown, missing, or otherwise un-supported kind code
    #[display("Unknown")]
    #[serde(rename = "Unknown")]
    Unknown,
}
/// Patent and patent-application enumerations for USPTO documents
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Patent {
    /// Granted patent document
    Granted {
        /// Country code of patent authority
        country_code: Option<CountryCode>,
        /// Kind code
        kind_code: KindCode,
        /// 1-7 character serial number
        serial_number: String,
    },
    /// Patent-application document
    Application {
        /// 1-7 character serial number
        serial_number: String,
        /// Application year (e.g., "2025")
        year: Option<String>,
    },
    /// Patent publication
    Publication {
        /// Country code of patent authority
        country_code: Option<CountryCode>,
        /// Kind code
        kind_code: KindCode,
        /// 1-7 character serial number
        serial_number: String,
        /// Publication year (e.g., "2025")
        year: Option<String>,
    },
}
impl From<&str> for CountryCode {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            | "US" => CountryCode::US,
            | "CN" => CountryCode::CN,
            | "DE" => CountryCode::DE,
            | "EP" => CountryCode::EP,
            | "JP" => CountryCode::JP,
            | "KR" => CountryCode::KR,
            | _ => CountryCode::US,
        }
    }
}
impl From<String> for CountryCode {
    fn from(s: String) -> Self {
        CountryCode::from(s.as_str())
    }
}
impl From<&str> for KindCode {
    fn from(s: &str) -> Self {
        match s {
            | "A1" => KindCode::A1,
            | "A2" => KindCode::A2,
            | "A9" => KindCode::A9,
            | "B1" => KindCode::B1,
            | "B2" => KindCode::B2,
            | "C1" => KindCode::C1,
            | "C2" => KindCode::C2,
            | "C3" => KindCode::C3,
            | "E" | "E1" => KindCode::E1,
            | "S" | "S1" => KindCode::S1,
            | "P1" => KindCode::P1,
            | "P2" => KindCode::P2,
            | "P3" => KindCode::P3,
            | "P4" => KindCode::P4,
            | "P9" => KindCode::P9,
            | "H" | "H1" => KindCode::H1,
            | _ => KindCode::Unknown,
        }
    }
}
impl From<String> for KindCode {
    fn from(s: String) -> Self {
        KindCode::from(s.as_str())
    }
}
impl KindCode {
    /// Return true if a kind code is a granted patent, false otherwise (e.g., application, publication)
    pub fn is_granted(&self) -> bool {
        match self {
            | KindCode::B1
            | KindCode::B2
            | KindCode::C1
            | KindCode::C2
            | KindCode::C3
            | KindCode::E1
            | KindCode::P2
            | KindCode::P3
            | KindCode::S1 => true,
            | _ => false,
        }
    }
}
impl Default for Patent {
    fn default() -> Self {
        Patent::Granted {
            country_code: Some(CountryCode::US),
            kind_code: KindCode::Unknown,
            serial_number: String::new(),
        }
    }
}
impl core::fmt::Display for Patent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            | Patent::Granted {
                country_code,
                serial_number,
                kind_code,
            } => {
                write!(f, "{} {serial_number} {kind_code}", country_code.clone().unwrap_or_default())
            }
            | Patent::Application { serial_number, year } => {
                if let Some(year) = year {
                    write!(f, "{year}/{serial_number}")
                } else {
                    write!(f, "{serial_number}")
                }
            }
            | Patent::Publication {
                country_code,
                serial_number,
                kind_code,
                year,
            } => {
                let country = country_code.clone().unwrap_or_default();
                if let Some(year) = year {
                    write!(f, "{country} {year}/{serial_number} {kind_code}")
                } else {
                    write!(f, "{country} {serial_number} {kind_code}")
                }
            }
        }
    }
}
impl From<&str> for Patent {
    fn from(s: &str) -> Self {
        Patent::parse(s).unwrap_or_default()
    }
}
impl From<String> for Patent {
    fn from(s: String) -> Self {
        Patent::from(s.as_str())
    }
}
impl Patent {
    /// Find all patent identifier values present in a string
    pub fn find_all(value: &str) -> Vec<Self> {
        let re = &RE_PATENT;
        re.find_iter(value).flat_map(|m| Patent::parse(m.unwrap().as_str())).collect::<Vec<_>>()
    }
    /// Check if a string is a valid patent identifier
    pub fn is_valid<S>(value: S) -> bool
    where
        S: AsRef<str>,
    {
        match Patent::parse(value.as_ref()) {
            | Some(result) => match result {
                | Patent::Granted {
                    serial_number, kind_code, ..
                } => !serial_number.is_empty() && kind_code != KindCode::Unknown,
                | Patent::Application { serial_number, .. } => !serial_number.is_empty(),
                | Patent::Publication {
                    serial_number, kind_code, ..
                } => !serial_number.is_empty() && kind_code != KindCode::Unknown,
            },
            | None => false,
        }
    }
    /// Parse a patent identifier into a `Patent` enumeration
    /// ### Note
    /// > Parsing is focused primarily of patents created after 02/01/2001
    pub fn parse<S>(value: S) -> Option<Self>
    where
        S: Into<String> + Clone,
    {
        let s: String = value.clone().into().chars().take(2).collect::<String>();
        match CountryCode::from(s) {
            | CountryCode::US => {
                fn preprocess<S>(value: S) -> String
                where
                    S: Into<String>,
                {
                    value.into().replace(" ", "").replace(",", "").replace("-", "").to_uppercase()
                }
                let pattern = format!("^{RE_PATENT_TEXT}$");
                let s: String = preprocess(value);
                let groups = ["country_code", "year", "serial_number", "kind_code"]
                    .into_iter()
                    .map(String::from)
                    .collect::<Vec<_>>();
                let lookup = regex_capture_lookup(pattern, s, groups);
                let country_code = lookup.get("country_code").cloned().map(CountryCode::from);
                let serial_number = lookup.get("serial_number").cloned().unwrap_or_default();
                let kind_code = match lookup.get("kind_code").cloned() {
                    | Some(value) => KindCode::from(value),
                    | None => KindCode::Unknown,
                };
                match kind_code {
                    | KindCode::B1
                    | KindCode::B2
                    | KindCode::C1
                    | KindCode::C2
                    | KindCode::C3
                    | KindCode::E1
                    | KindCode::P2
                    | KindCode::P3
                    | KindCode::S1 => Some(Patent::Granted {
                        country_code,
                        serial_number,
                        kind_code,
                    }),
                    | KindCode::A1 | KindCode::A2 | KindCode::A9 | KindCode::P1 | KindCode::P4 | KindCode::P9 => {
                        let year = lookup.get("year").cloned();
                        Some(Patent::Publication {
                            country_code,
                            serial_number,
                            kind_code,
                            year,
                        })
                    }
                    | _ => None,
                }
            }
            | _ => unimplemented!("Only US patents are supported"),
        }
    }
}
