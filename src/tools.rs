//! Batch tag tools, mirroring the original's Tools menu:
//! track # formatting, auto fill, swaps, artist field tools and the
//! remove-spaces helpers.

use crate::data::{Field, TagFields};

/// "Format Track # to ..." targets, from the original's menu
/// (x, 0x, 00x, xx/xx, xx).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TrackFormat {
    X,
    ZeroX,
    ZeroZeroX,
    XxXx,
    Xx,
}

impl TrackFormat {
    pub fn label(self) -> &'static str {
        match self {
            TrackFormat::X => "Format Track # to x",
            TrackFormat::ZeroX => "Format Track # to 0x",
            TrackFormat::ZeroZeroX => "Format Track # to 00x",
            TrackFormat::XxXx => "Format Track # to xx/xx",
            TrackFormat::Xx => "Format Track # to xx",
        }
    }

    /// Rules from the person's guide (Tools > Format Track #):
    /// - x    track number as a plain number: "01" -> "1"
    /// - 0x   two digits: "1" -> "01"
    /// - 00x  three digits: "1" -> "001"
    /// - xx/xx  track number with two digits, then "/" and the number of
    ///        checked files (the tracks in the EP or LP), two digits: "01/01"
    /// - xx   two digits and the total removed: "1/12" -> "01"
    /// x, 0x and 00x keep the total as it is. A value that is not a plain
    /// number (or is empty) is left as it is.
    pub fn apply(self, track: &str, total: &str, checked_total: u32) -> (String, String) {
        let number = |width: usize| pad_number(&drop_total(track), width);
        match self {
            TrackFormat::X => (number(1), total.to_string()),
            TrackFormat::ZeroX => (number(2), total.to_string()),
            TrackFormat::ZeroZeroX => (number(3), total.to_string()),
            TrackFormat::XxXx => (number(2), format!("{checked_total:02}")),
            TrackFormat::Xx => (number(2), String::new()),
        }
    }
}

/// Pads a plain number to at least `width` digits. Anything else is returned trimmed.
fn pad_number(s: &str, width: usize) -> String {
    let s = s.trim();
    if s.is_empty() || !s.chars().all(|c| c.is_ascii_digit()) {
        return s.to_string();
    }
    match s.parse::<u64>() {
        Ok(n) => format!("{n:0width$}"),
        Err(_) => s.to_string(),
    }
}

/// Copy(1, Pos("/", s)-1) then Trim: everything from the first "/" is dropped.
fn drop_total(s: &str) -> String {
    match s.find('/') {
        Some(p) => s[..p].trim().to_string(),
        None => s.trim().to_string(),
    }
}

/// Auto Fill Track #. The dialog is built in sub_caa73c @ 0xcaa73c; the
/// Yes handler is `thunk_FUN_00cab32e` @ 0xcab300 (Ghidra, Deep mode). It
/// asks "Auto fill track # for all selected files?" with "Start from:"
/// (spin, max 10000) and "Use xx/xx format" (Config `Use_Xx_Xx`);
/// "Don't add leading zero when autofilling track #" is Config
/// `DontAddLeadingZeroToTrackNr`, default off, at form+0xc8a.
///
/// Deep decompile of the Yes handler (0xcab300, lines 32-50) shows three
/// independent zero-prepends, each gated by that option: one when the
/// counter is below 10, one when the folder scope holds >= 100 files and
/// the counter is below 100 (sub_caa618 @ 0xcaa618), one when the scope
/// holds >= 1000 files and the counter is below 1000 (sub_caa6a8 @
/// 0xcaa6a8). So the width is 2/3/4 by SCOPE size, not by how many files
/// are checked. The counter is reset to 0 when the folder scope ends
/// (sub_722504 @ 0x722504); the first file starts at `start`.
///
/// With "xx/xx" the appended total is the SELECTED count padded to two
/// digits only (literals "/0" 0xcab708 and "/" 0xcab71c).
pub struct AutoFill {
    pub start: u32,
    pub use_total: bool,
    pub leading_zero: bool,
}

/// `counter` is the current track number (already including Start from);
/// `scope_count` is the number of files in the current folder and drives
/// the zero width; `selected_count` is only used for the "/total" suffix.
pub fn auto_fill_track(
    counter: u32,
    scope_count: u32,
    selected_count: u32,
    opts: &AutoFill,
) -> (String, String) {
    // Two digits below 100 checked files in the folder, three below 1000,
    // else four; the original prepends one "0" per branch (0xcab300: the
    // counter < 10 branch, then the scope >= 100 and >= 1000 branches),
    // which is exactly zero-padding to this width.
    let width: usize = if scope_count >= 1000 {
        4
    } else if scope_count >= 100 {
        3
    } else {
        2
    };
    let track = if opts.leading_zero {
        format!("{counter:0width$}")
    } else {
        counter.to_string()
    };
    // The total is only ever padded to two digits, whatever the count.
    let total = if opts.use_total {
        if selected_count < 10 {
            format!("0{selected_count}")
        } else {
            selected_count.to_string()
        }
    } else {
        String::new()
    };
    (track, total)
}

/// Swap two fields, as the original's "Swap Tag Fields" menu does.
pub fn swap_fields(fields: &mut TagFields, a: Field, b: Field) {
    let va = fields.get(a).to_string();
    let vb = fields.get(b).to_string();
    fields.set(a, &vb);
    fields.set(b, &va);
}

/// Copy one field into another ("Copy Text from One Tag Field to Another").
pub fn copy_field(fields: &mut TagFields, from: Field, to: Field) {
    let v = fields.get(from).to_string();
    fields.set(to, &v);
}

/// Artist field tools, each matching the original's menu captions.
#[derive(Clone, Copy)]
pub enum ArtistTool {
    RemoveLeadingThe,
    TheArtistToArtistThe,
    ArtistTheToTheArtist,
    NameSurnameToSurnameName,
    SurnameNameToNameSurname,
}

impl ArtistTool {
    pub fn label(self) -> &'static str {
        match self {
            ArtistTool::RemoveLeadingThe => "Remove Leading \"The\" from Artist Field",
            ArtistTool::TheArtistToArtistThe => "Format \"The Artist\" to \"Artist, The\"",
            ArtistTool::ArtistTheToTheArtist => "Format \"Artist, The\" to \"The Artist\"",
            ArtistTool::NameSurnameToSurnameName => "Format Artist Field to \"Surname, Name\"",
            ArtistTool::SurnameNameToNameSurname => "Format Artist Field to \"Name Surname\"",
        }
    }

    pub fn apply(self, artist: &str) -> String {
        let trimmed = artist.trim();
        match self {
            ArtistTool::RemoveLeadingThe => {
                let lower = trimmed.to_lowercase();
                if lower.strip_prefix("the ").is_some() {
                    let cut = "the ".len();
                    trimmed[cut..].trim_start().to_string()
                } else {
                    trimmed.to_string()
                }
            }
            ArtistTool::TheArtistToArtistThe => {
                let lower = trimmed.to_lowercase();
                if lower.strip_prefix("the ").is_some() {
                    let cut = "the ".len();
                    let name = trimmed[cut..].trim_start();
                    if name.is_empty() {
                        trimmed.to_string()
                    } else {
                        format!("{name}, The")
                    }
                } else {
                    trimmed.to_string()
                }
            }
            ArtistTool::ArtistTheToTheArtist => {
                if let Some(name) = trimmed.strip_suffix(", The").or_else(|| trimmed.strip_suffix(", the")) {
                    let n = name.trim();
                    if n.is_empty() {
                        trimmed.to_string()
                    } else {
                        format!("The {n}")
                    }
                } else {
                    trimmed.to_string()
                }
            }
            ArtistTool::NameSurnameToSurnameName => {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    let surname = parts[parts.len() - 1];
                    let given = parts[..parts.len() - 1].join(" ");
                    format!("{surname}, {given}")
                } else {
                    trimmed.to_string()
                }
            }
            ArtistTool::SurnameNameToNameSurname => {
                if let Some(pos) = trimmed.find(", ") {
                    let surname = &trimmed[..pos];
                    let given = trimmed[pos + 2..].trim();
                    if !surname.is_empty() && !given.is_empty() {
                        return format!("{given} {surname}");
                    }
                }
                trimmed.to_string()
            }
        }
    }
}

/// "Remove Trailing Spaces from All Tag Fields" — trailing spaces only.
pub fn remove_trailing_spaces(fields: &mut TagFields) {
    for f in crate::data::ALL_FIELDS {
        let v = fields.get(*f).trim_end().to_string();
        fields.set(*f, &v);
    }
}

/// "Remove Leading and Trailing Spaces from All Tag Fields".
pub fn remove_all_spaces(fields: &mut TagFields) {
    for f in crate::data::ALL_FIELDS {
        let v = fields.get(*f).trim().to_string();
        fields.set(*f, &v);
    }
}

/// The rating combo values, in the original's order. sub_cdaa90 (0xcdaa90)
/// fills the tag editor's combo from .itext strings 0xcdacb0..0xcdad3c in the
/// order 5 Stars, 4 Stars, 3 Stars, 2 Stars, 1 Star, Unrated (fresh decompile
/// ev_8a559dae…, sub_cdaa90: literals 0xcdacb0..0xcdad3c, same order as the
/// resourcestring table 0xdf79e4-0xdf7a32). Index 0 is 5 Stars.
pub const RATINGS: &[&str] = &["5 Stars", "4 Stars", "3 Stars", "2 Stars", "1 Star", "Unrated"];

/// Convert a star count (0 = Unrated .. 5 = 5 Stars) into the combo label.
/// Index 0 of RATINGS is 5 Stars, so the label for `stars` is at 5 - stars.
pub fn rating_from_stars(stars: usize) -> &'static str {
    RATINGS[5 - stars.min(5)]
}

/// Convert a combo label into a star count (0 = Unrated .. 5 = 5 Stars).
pub fn stars_from_rating(value: &str) -> usize {
    for (i, r) in RATINGS.iter().enumerate() {
        if value.eq_ignore_ascii_case(r) {
            return 5 - i;
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rating_order_and_mapping_match_the_original_combo() {
        // sub_cdaa90 fills the combo in this order; resourcestring table
        // 0xdf79e4-0xdf7a32 agrees.
        assert_eq!(
            RATINGS,
            &["5 Stars", "4 Stars", "3 Stars", "2 Stars", "1 Star", "Unrated"]
        );
        // Star count (0 = Unrated .. 5 = 5 Stars) round-trips through labels.
        for stars in 0..=5 {
            let label = rating_from_stars(stars);
            assert_eq!(stars_from_rating(label), stars, "label {label}");
        }
        assert_eq!(rating_from_stars(5), "5 Stars");
        assert_eq!(rating_from_stars(0), "Unrated");
        assert_eq!(stars_from_rating("not a rating"), 0);
    }

    #[test]
    fn format_track_rules_follow_the_guide() {
        // x: plain number, no leading zeros; total kept.
        assert_eq!(TrackFormat::X.apply("05", "12", 3), ("5".into(), "12".into()));
        assert_eq!(TrackFormat::X.apply("007", "", 3), ("7".into(), "".into()));
        assert_eq!(TrackFormat::X.apply("10", "", 3), ("10".into(), "".into()));
        assert_eq!(TrackFormat::X.apply("1", "", 3), ("1".into(), "".into()));

        // 0x: two digits; total kept.
        assert_eq!(TrackFormat::ZeroX.apply("1", "12", 3), ("01".into(), "12".into()));
        assert_eq!(TrackFormat::ZeroX.apply("01", "", 3), ("01".into(), "".into()));
        assert_eq!(TrackFormat::ZeroX.apply("45", "", 3), ("45".into(), "".into()));
        assert_eq!(TrackFormat::ZeroX.apply("105", "", 3), ("105".into(), "".into()));

        // 00x: three digits; total kept.
        assert_eq!(TrackFormat::ZeroZeroX.apply("1", "", 3), ("001".into(), "".into()));
        assert_eq!(TrackFormat::ZeroZeroX.apply("45", "9", 3), ("045".into(), "9".into()));
        assert_eq!(TrackFormat::ZeroZeroX.apply("105", "", 3), ("105".into(), "".into()));

        // xx/xx: track two digits, total = checked count, two digits.
        assert_eq!(TrackFormat::XxXx.apply("1", "", 12), ("01".into(), "12".into()));
        assert_eq!(TrackFormat::XxXx.apply("5", "9", 5), ("05".into(), "05".into()));
        assert_eq!(TrackFormat::XxXx.apply("1/9", "", 5), ("01".into(), "05".into()));

        // xx: two digits, total removed.
        assert_eq!(TrackFormat::Xx.apply("1", "12", 3), ("01".into(), "".into()));
        assert_eq!(TrackFormat::Xx.apply("5/12", "", 3), ("05".into(), "".into()));
        assert_eq!(TrackFormat::Xx.apply("", "", 3), ("".into(), "".into()));
    }

    #[test]
    fn auto_fill_widths_and_totals() {
        let opts = AutoFill { start: 1, use_total: true, leading_zero: true };
        // Two digits when the FOLDER scope is below 100 files, whatever
        // the number of checked files (sub_722504/sub_caa618 at 0xcab300).
        assert_eq!(auto_fill_track(1, 12, 12, &opts), ("01".into(), "12".into()));
        assert_eq!(auto_fill_track(9, 12, 12, &opts), ("09".into(), "12".into()));
        assert_eq!(auto_fill_track(10, 12, 3, &opts), ("10".into(), "03".into()));
        // Three digits from 100 files in scope, four from 1000.
        assert_eq!(auto_fill_track(1, 150, 150, &opts), ("001".into(), "150".into()));
        assert_eq!(auto_fill_track(1, 1500, 1500, &opts), ("0001".into(), "1500".into()));
        // The total is the selected count, padded to two only.
        assert_eq!(auto_fill_track(1, 5, 5, &opts), ("01".into(), "05".into()));
        // Start from is honored by the caller's counter, not here.
        let from5 = AutoFill { start: 5, use_total: false, leading_zero: true };
        assert_eq!(auto_fill_track(5, 9, 9, &from5), ("05".into(), "".into()));
        // DontAddLeadingZeroToTrackNr on: no padding at all.
        let no0 = AutoFill { start: 1, use_total: true, leading_zero: false };
        assert_eq!(auto_fill_track(1, 12, 12, &no0), ("1".into(), "12".into()));
        // At a folder boundary the caller resets the counter to 0
        // (sub_722504) and then increments before formatting, so the first
        // file of the new folder gets "01", not the dialog's Start-from.
        assert_eq!(auto_fill_track(0, 12, 12, &opts), ("00".into(), "12".into()));
        assert_eq!(auto_fill_track(1, 12, 12, &opts), ("01".into(), "12".into()));
    }
}
