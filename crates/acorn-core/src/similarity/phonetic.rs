//! Explicit language-specific phonetic encodings.
use super::{SimilarityLimits, SimilarityResult};
use crate::prelude::alloc::{String, ToString, Vec};

/// Extension methods for portable natural-language phonetic processing.
pub trait NaturalLanguage: AsRef<str> {
    /// Return the Cologne phonetic code for one uppercase character in context.
    fn cologne_code(&self, character: char, previous: Option<char>, next: Option<char>, initial: bool) -> &'static str {
        match character {
            | 'A' | 'E' | 'I' | 'J' | 'O' | 'U' | 'Y' => "0",
            | 'B' => "1",
            | 'P' if next != Some('H') => "1",
            | 'D' | 'T' if !matches!(next, Some('C' | 'S' | 'Z')) => "2",
            | 'F' | 'P' | 'V' | 'W' => "3",
            | 'G' | 'K' | 'Q' => "4",
            | 'C' if initial && matches!(next, Some('A' | 'H' | 'K' | 'L' | 'O' | 'Q' | 'R' | 'U' | 'X')) => "4",
            | 'C' if !initial && !matches!(previous, Some('S' | 'Z')) && matches!(next, Some('A' | 'H' | 'K' | 'O' | 'Q' | 'U' | 'X')) => "4",
            | 'X' if matches!(previous, Some('C' | 'K' | 'Q')) => "8",
            | 'X' => "48",
            | 'L' => "5",
            | 'M' | 'N' => "6",
            | 'R' => "7",
            | 'C' | 'S' | 'Z' => "8",
            | _ => "",
        }
    }
    /// Encode the text with the German-language Cologne phonetics rules.
    fn cologne_phonetic(&self, limits: SimilarityLimits) -> SimilarityResult<String> {
        let value = self.as_ref();
        limits.check_input(value, "input").map(|_| {
            let letters = value
                .chars()
                .flat_map(char::to_uppercase)
                .filter_map(|character| match character {
                    | 'Ä' => Some('A'),
                    | 'Ö' => Some('O'),
                    | 'Ü' => Some('U'),
                    | 'ß' | 'ẞ' => Some('S'),
                    | character if character.is_ascii_alphabetic() => Some(character),
                    | _ => None,
                })
                .collect::<Vec<_>>();
            letters
                .iter()
                .enumerate()
                .flat_map(|(index, character)| {
                    self.cologne_code(
                        *character,
                        index.checked_sub(1).and_then(|index| letters.get(index)).copied(),
                        letters.get(index.saturating_add(1)).copied(),
                        index == 0,
                    )
                    .chars()
                })
                .fold((String::new(), None), |(mut encoded, previous), code| {
                    if previous != Some(code) && (code != '0' || encoded.is_empty()) {
                        encoded.push(code);
                    }
                    (encoded, Some(code))
                })
                .0
        })
    }
    /// Return whether an optional uppercase character is an ASCII vowel.
    fn is_vowel(&self, character: Option<char>) -> bool {
        matches!(character, Some('A' | 'E' | 'I' | 'O' | 'U'))
    }
    /// Encode the text with Lawrence Philips' original English-language Metaphone rules.
    fn metaphone(&self, limits: SimilarityLimits) -> SimilarityResult<String> {
        let value = self.as_ref();
        limits.check_input(value, "input").map(|_| {
            let mut letters = self.to_ascii();
            match letters.as_slice() {
                | ['K' | 'G' | 'P', 'N', ..] | ['A', 'E', ..] | ['W', 'R', ..] => {
                    letters.remove(0);
                }
                | ['W', 'H', ..] => {
                    letters.remove(1);
                }
                | ['X', ..] => {
                    if let Some(first) = letters.first_mut() {
                        *first = 'S';
                    }
                }
                | _ => {}
            }
            (0..letters.len()).fold(String::new(), |mut encoded, index| {
                let previous = index.checked_sub(1).and_then(|index| letters.get(index)).copied();
                let current = letters.get(index).copied().unwrap_or_default();
                let next = letters.get(index.saturating_add(1)).copied();
                let next_two = letters.get(index.saturating_add(2)).copied();
                let duplicate = previous == Some(current) && current != 'C';
                if !duplicate {
                    match current {
                        | 'A' | 'E' | 'I' | 'O' | 'U' if index == 0 => encoded.push(current),
                        | 'B' if previous != Some('M') || index.saturating_add(1) != letters.len() => encoded.push('B'),
                        | 'C' if next == Some('H') => encoded.push('X'),
                        | 'C' if next == Some('I') && next_two == Some('A') => encoded.push('X'),
                        | 'C' if matches!(next, Some('I' | 'E' | 'Y')) => encoded.push('S'),
                        | 'D' if next == Some('G') && matches!(next_two, Some('E' | 'I' | 'Y')) => encoded.push('J'),
                        | 'D' => encoded.push('T'),
                        | 'F' | 'J' | 'L' | 'M' | 'N' | 'R' => encoded.push(current),
                        | 'G' if next == Some('H') && !self.is_vowel(previous) => {}
                        | 'G' if matches!(next, Some('E' | 'I' | 'Y')) => encoded.push('J'),
                        | 'C' | 'G' | 'Q' => encoded.push('K'),
                        | 'H' if self.is_vowel(next) && !self.is_vowel(previous) && !matches!(previous, Some('C' | 'G' | 'P' | 'S' | 'T')) => {
                            encoded.push('H');
                        }
                        | 'K' if previous != Some('C') => encoded.push('K'),
                        | 'P' if next == Some('H') => encoded.push('F'),
                        | 'P' => encoded.push('P'),
                        | 'S' if next == Some('H') || (matches!(next, Some('I')) && matches!(next_two, Some('A' | 'O'))) => encoded.push('X'),
                        | 'S' | 'Z' => encoded.push('S'),
                        | 'T' if matches!(next, Some('I')) && matches!(next_two, Some('A' | 'O')) => encoded.push('X'),
                        | 'T' if next == Some('H') => encoded.push('0'),
                        | 'T' if !(next == Some('C') && next_two == Some('H')) => encoded.push('T'),
                        | 'V' => encoded.push('F'),
                        | 'W' | 'Y' if self.is_vowel(next) => encoded.push(current),
                        | 'X' => encoded.push_str("KS"),
                        | _ => {}
                    }
                }
                encoded
            })
        })
    }
    /// Encode the text with American Soundex.
    fn soundex(&self, limits: SimilarityLimits) -> SimilarityResult<String> {
        let value = self.as_ref();
        limits.check_input(value, "input").map(|_| {
            let letters = self.to_ascii();
            letters.first().map_or_else(String::new, |first| {
                let initial_code = self.soundex_code(*first);
                let (mut encoded, _) = letters
                    .iter()
                    .skip(1)
                    .fold((first.to_string(), initial_code), |(mut encoded, previous), character| {
                        let code = self.soundex_code(*character);
                        if code.is_ascii_digit() && code != '0' && code != previous && encoded.len() < 4 {
                            encoded.push(code);
                        }
                        let previous = match code {
                            | '-' => previous,
                            | _ => code,
                        };
                        (encoded, previous)
                    });
                while encoded.len() < 4 {
                    encoded.push('0');
                }
                encoded
            })
        })
    }
    /// Return the American Soundex class for one uppercase character.
    fn soundex_code(&self, character: char) -> char {
        match character {
            | 'B' | 'F' | 'P' | 'V' => '1',
            | 'C' | 'G' | 'J' | 'K' | 'Q' | 'S' | 'X' | 'Z' => '2',
            | 'D' | 'T' => '3',
            | 'L' => '4',
            | 'M' | 'N' => '5',
            | 'R' => '6',
            | 'H' | 'W' => '-',
            | _ => '0',
        }
    }
    /// Convert the text to uppercase ASCII letters for English phonetic encoders.
    fn to_ascii(&self) -> Vec<char> {
        self.as_ref()
            .chars()
            .flat_map(char::to_uppercase)
            .filter(|character| character.is_ascii_alphabetic())
            .collect()
    }
}
impl NaturalLanguage for &str {}
impl NaturalLanguage for String {}
/// Encode German-language text with the Cologne phonetics rules.
pub fn cologne_phonetic(value: &str, limits: SimilarityLimits) -> SimilarityResult<String> {
    value.cologne_phonetic(limits)
}
/// Encode English-language text with Lawrence Philips' original Metaphone rules.
pub fn metaphone(value: &str, limits: SimilarityLimits) -> SimilarityResult<String> {
    value.metaphone(limits)
}
/// Encode English-language text with American Soundex.
pub fn soundex(value: &str, limits: SimilarityLimits) -> SimilarityResult<String> {
    value.soundex(limits)
}
