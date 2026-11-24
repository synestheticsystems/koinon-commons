use ankurah::{envelope::EnvelopeComponent, Model};
use serde::{Deserialize, Serialize};

use crate::mime_type::MimeTypes;

/// How a registry recognises a kind of printer, and what it knows of it: that a printer whose
/// driver is named like this takes documents of these formats as it stands, sent to it
/// untouched: the formats it prints with no setting changed, or the one its driver's name states.
/// A printer does not reliably say which formats it takes, and the print system does not ask it,
/// so a registry reads a printer's `driver` against these records: the test print is written in
/// the first of its formats, and a caller is told them all. A printer whose driver no record names
/// takes no format of its own that anyone knows of, and its test print is a PDF when its document
/// formats list `application/pdf`.
///
/// A printer's own language has no registered MIME type, so each is named as an unregistered
/// type, `application/x-` followed by the language's name in lower case: `application/x-zpl2`
/// for ZPL II, the ZPL that current printers speak, and `application/x-epl2`. A language with
/// two forms that differ is named by the form, since the type says which bytes to write.
///
/// The records stand in an order, the first whose text the driver's name contains deciding, as
/// [`names_driver`] compares them: on a registry, the order of the Ankurah Commons'
/// `FractionalPosition`, on the same entity, and in a `Catalog`, the order of its rows. So a record
/// for a model's longer name stands before one for its shorter name, which the longer contains, as
/// "LP 2844-Z" contains "LP 2844".
///
/// Who added it is the Ankurah Commons' `Created`, and its withdrawal `Closed`, on the same
/// entity. A record the registry started with was added by no one and carries no `Created`.
#[derive(Model, EnvelopeComponent, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[model(label = "catalog.printer_fingerprint")]
#[model(id = "HER0P_9s7PPq1hpuHBxuqG1HZuvxqoEn56XdVpv9lu0")]
#[model(no_ffi)]
pub struct PrinterFingerprint {
    /// What the driver's name contains, such as "LP 2844" or "ZPL".
    #[active_type(LWW)]
    #[property(id = "DvWs3dTp3d55NySSJQcnYF0odq--KY_i-h6Tv44f6a0")]
    #[envelope(number = 0)]
    pub driver: String,
    /// The formats, as MIME types, in the order a registry prefers them.
    #[active_type(LWW)]
    #[property(id = "GSn2A9b_IVN5BHjydBW9B8Sgkx0N-Nd2S6-7sje0cFg")]
    #[envelope(number = 1)]
    pub document_formats: MimeTypes,
}

/// Whether a driver's name contains `text`, as a `PrinterFingerprint` record's text is compared
/// with it: by their letters and digits alone, without regard to case, with the text beginning
/// where a word of the name begins and ending where one ends. A word is a run of letters or a run
/// of digits, so "TLP2844" and "TLP 2844" read alike, and "LP 2844" is in neither. A text with no
/// letter or digit names no driver.
pub fn names_driver(text: &str, driver: &str) -> bool {
    let text = Compared::new(text).characters;
    let driver = Compared::new(driver);
    !text.is_empty()
        && driver
            .characters
            .windows(text.len())
            .enumerate()
            .any(|(start, window)| window == text.as_slice() && driver.word_edges[start] && driver.word_edges[start + text.len()])
}

/// A name as `names_driver` reads it.
struct Compared {
    /// Its letters and digits, in lower case.
    characters: Vec<char>,
    /// For each place before, between and after `characters`, whether a word of the name ends
    /// or begins there.
    word_edges: Vec<bool>,
}

impl Compared {
    fn new(name: &str) -> Self {
        let mut characters = Vec::new();
        let mut word_edges = Vec::new();
        // Whether the last letter or digit was a digit, and whether anything else followed it.
        let mut last_was_digit = None;
        let mut parted = false;
        for character in name.chars() {
            if !character.is_alphanumeric() {
                parted = true;
                continue;
            }
            let digit = character.is_numeric();
            let begins_word = match last_was_digit {
                None => true,
                Some(was_digit) => parted || was_digit != digit,
            };
            word_edges.push(begins_word);
            // A letter that lowers into several characters is still one place of the name.
            for (index, lowered) in character.to_lowercase().enumerate() {
                if index > 0 {
                    word_edges.push(false);
                }
                characters.push(lowered);
            }
            last_was_digit = Some(digit);
            parted = false;
        }
        word_edges.push(true);
        Compared { characters, word_edges }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_text_is_found_where_words_of_the_name_begin_and_end() {
        assert!(names_driver("LP 2844", "ZDesigner LP 2844"));
        assert!(names_driver("tlp 2844", "Zebra TLP2844"), "case and spaces count for nothing");
        assert!(names_driver("Z4MPlus", "Zebra Z4M Plus"), "nor does where the spaces fall");
        assert!(!names_driver("LP 2844", "Zebra TLP 2844"), "LP is no word of TLP");
        assert!(!names_driver("LP 2844", "Zebra LP 28445"), "2844 is no word of 28445");
        assert!(!names_driver("LP 2844-Z", "LP 2844"), "a text longer than the name");
        assert!(!names_driver(" - ", "Zebra LP 2844"), "a text with no letter or digit");
    }
}
