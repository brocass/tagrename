//! The mask engine for "Rename Files" and "Get Tags from File Name".
//!
//! The tokens and every formula below were read from the original's code
//! (see MECHANICS.md): the name builder is sub_c9f784 at 0xc9f784 plus its
//! continuation sub_ca0821 (tokens, escaping, padding, the case options),
//! and the file-name parser is sub_ca3598 at 0xca3598, called by the
//! file-to-tag write engine sub_ca4d14 (handler actMainFunctions_FileToTag
//! WriteExecute at 0xcaf738).
//! %1 Artist, %2 Title, %3 Album, %4 Year, %5 Genre, %6 Track #,
//! %7 File name, %8 File number, %9 Comment, %0 Bitrate, %t Play time,
//! %b BPM, %# Disc #, %aa Album Artist, %co Conductor, %cp Composer,
//! %rt Rating, %md Mood, %lb Label, %rb Remixed by, %key Initial Key,
//! %swt Show Title, %eid Episode ID, %snr Season Number,
//! %enr Episode Number, %rnd Random Number, %ext File Ext., %khz Hz,
//! %dbit Bit Depth, %enc Encoder; and in file-name parsing %d skips a part.

use crate::data::{Field, MusicFile};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Case {
    /// The Case checkbox is off: leave text as it is.
    Off,
    Lower,
    Upper,
    CapitalizeFirstLetter,
    CapitalizeFirstWord,
}

pub const CASE_OPTIONS: &[&str] = &[
    "lower",
    "UPPER",
    "Capitalize First Letter",
    "Capitalize first word",
];

impl Case {
    pub fn from_index(i: usize) -> Case {
        match i {
            0 => Case::Lower,
            1 => Case::Upper,
            2 => Case::CapitalizeFirstLetter,
            _ => Case::CapitalizeFirstWord,
        }
    }

    pub fn apply(self, s: &str) -> String {
        match self {
            Case::Off => s.to_string(),
            Case::Lower => s.to_lowercase(),
            Case::Upper => s.to_uppercase(),
            // Original sub_6da398 (radio "Capitalize First Letter"):
            // lower-case everything, upper-case the first character, then
            // upper-case every character that follows a separator from the
            // runtime table at *(0xcf24f0)+0x44. Its default (Config key
            // `CapitalizeConfiguration`, literal 0xc98e84) is
            // `_ ()[]-{}"&#.1234567890\/` — digits included, so
            // "track 2 name" becomes "Track 2 Name".
            Case::CapitalizeFirstLetter => {
                const SEPARATORS: &str = "_ ()[]-{}\"&#.1234567890\\/";
                let mut out: Vec<char> = s.to_lowercase().chars().collect();
                let mut made_upper = false;
                let mut after_separator = false;
                for c in out.iter_mut() {
                    if SEPARATORS.contains(*c) {
                        after_separator = true;
                        continue;
                    }
                    if !made_upper || after_separator {
                        *c = c.to_uppercase().next().unwrap_or(*c);
                        made_upper = true;
                        after_separator = false;
                    }
                }
                out.into_iter().collect()
            }
            // Original sub_6da4d8 (radio "Capitalize first word"):
            // lower-case everything, then upper-case only the first
            // character.
            Case::CapitalizeFirstWord => {
                let mut out = s.to_lowercase();
                if let Some(pos) = out.find(|c: char| c.is_alphabetic()) {
                    let upper: String = out[pos..].chars().take(1).flat_map(|c| c.to_uppercase()).collect();
                    out.replace_range(pos..pos + 1, &upper);
                }
                out
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Token {
    Tag(Field),
    FileName,
    FileNumber,
    PlayTime,
    Bitrate,
    SampleRate,
    BitDepth,
    Encoder,
    FileExt,
    Random,
    /// File-name parsing only: capture and discard.
    Skip,
}

fn lookup_token(word: &str) -> Option<Token> {
    use Field::*;
    Some(match word {
        "1" => Token::Tag(Artist),
        "2" => Token::Tag(Title),
        "3" => Token::Tag(Album),
        "4" => Token::Tag(Year),
        "5" => Token::Tag(Genre),
        "6" => Token::Tag(Track),
        "7" => Token::FileName,
        "8" => Token::FileNumber,
        "9" => Token::Tag(Comment),
        "0" => Token::Bitrate,
        "t" => Token::PlayTime,
        "b" => Token::Tag(Bpm),
        "#" => Token::Tag(Disc),
        "aa" => Token::Tag(AlbumArtist),
        "co" => Token::Tag(Conductor),
        "cp" => Token::Tag(Composer),
        "rt" => Token::Tag(Rating),
        "md" => Token::Tag(Mood),
        "lb" => Token::Tag(Label),
        "rb" => Token::Tag(RemixedBy),
        "key" => Token::Tag(InitialKey),
        // The original substitutes these from the item's podcast/TV fields
        // (sub_c9f784: %swt from +0x12c, %eid from +0x130, %snr from +0x134,
        // %enr from +0x138). They now have their own fields, so the tokens
        // use the right values instead of standing in on other tags.
        "swt" => Token::Tag(ShowTitle),
        "eid" => Token::Tag(EpisodeId),
        "snr" => Token::Tag(SeasonNumber),
        "enr" => Token::Tag(EpisodeNumber),
        "rnd" => Token::Random,
        "ext" => Token::FileExt,
        "khz" => Token::SampleRate,
        "dbit" => Token::BitDepth,
        "enc" => Token::Encoder,
        "d" => Token::Skip,
        _ => return None,
    })
}

#[derive(Clone, Debug)]
pub enum Part {
    Literal(String),
    Var(Token),
}

/// Split the mask into literals and variables, longest tokens first.
pub fn parse_mask(mask: &str) -> Vec<Part> {
    let mut parts = Vec::new();
    let mut literal = String::new();
    let chars: Vec<char> = mask.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '%' && i + 1 < chars.len() {
            // Try the longest token word that matches.
            let mut matched: Option<(usize, Token)> = None;
            for len in (1..=4).rev() {
                if i + 1 + len <= chars.len() {
                    let word: String = chars[i + 1..i + 1 + len].iter().collect();
                    if let Some(t) = lookup_token(&word) {
                        matched = Some((len, t));
                        break;
                    }
                }
            }
            if let Some((len, token)) = matched {
                if !literal.is_empty() {
                    parts.push(Part::Literal(std::mem::take(&mut literal)));
                }
                parts.push(Part::Var(token));
                i += 1 + len;
                continue;
            }
        }
        literal.push(chars[i]);
        i += 1;
    }
    if !literal.is_empty() {
        parts.push(Part::Literal(literal));
    }
    parts
}

/// True when the mask contains at least one variable.
pub fn mask_has_variables(mask: &str) -> bool {
    parse_mask(mask).iter().any(|p| matches!(p, Part::Var(_)))
}

/// %8 File number (sub_c9f784 at 0xca05ab): the 1-based checked-file
/// counter that sub_cae988 passes in (incremented at 0xcaeb5b), prefixed
/// with "0" (0xca15c4) through sub_40b0b4 only when it is below 10 —
/// `cmp esi,0xa` / `jge` in the disassembly; there is no 3-digit rule.
fn pad_number(n: u32) -> String {
    if n < 10 {
        format!("0{n}")
    } else {
        n.to_string()
    }
}

/// The original's Random (sub_406cc8 at 0x406cc8): the Delphi LCG
/// seed := seed * 0x8088405 + 1; result := (Range * seed) shr 32.
/// The seed lives for the whole program run, exactly like the original's
/// global RandSeed at 0xcdf008.
fn delphi_random(range: u32) -> u32 {
    use std::cell::RefCell;
    thread_local! {
        static RAND_SEED: RefCell<u32> = const { RefCell::new(0) };
    }
    RAND_SEED.with(|s| {
        let mut seed = *s.borrow();
        seed = seed.wrapping_mul(0x0808_8405).wrapping_add(1);
        *s.borrow_mut() = seed;
        ((u64::from(range) * u64::from(seed)) >> 32) as u32
    })
}

/// %rnd Random Number (fresh decompile ev_57bf77ec…, sub_c9f784 lines
/// 833-885): with at most 0x63 (99) files in the list it is
/// Random(0x62=98)+1 zero-padded to 2 digits, otherwise Random(0x3e6=998)+1
/// zero-padded to 3 digits ("0" 0xca15c4, "00" 0xca15d4).
fn random_number(file_total: u32) -> String {
    let n = if file_total <= 99 {
        delphi_random(98) + 1
    } else {
        delphi_random(998) + 1
    };
    let s = n.to_string();
    if file_total <= 99 {
        if s.len() < 2 {
            format!("0{s}")
        } else {
            s
        }
    } else if s.len() < 3 {
        format!("{}{s}", "0".repeat(3 - s.len()))
    } else {
        s
    }
}

/// %t Play time (sub_640954 at 0x640954, then ":" -> "-" in sub_ca0821):
/// hours are omitted when 0 ("0h:" up to 9, "h:" above), minutes and
/// seconds are always two digits ("00:"/"00" when zero), and the colons
/// are swapped for dashes so the text is legal in a file name.
fn mask_play_time(secs_f: f64) -> String {
    let secs = secs_f.round() as u64;
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    let mut out = String::new();
    if h != 0 {
        if h <= 9 {
            out.push_str(&format!("0{h}:"));
        } else {
            out.push_str(&format!("{h}:"));
        }
    }
    if m != 0 {
        if m <= 9 {
            out.push_str(&format!("0{m}:"));
        } else {
            out.push_str(&format!("{m}:"));
        }
    } else {
        out.push_str("00:");
    }
    if s != 0 {
        if s <= 9 {
            out.push_str(&format!("0{s}"));
        } else {
            out.push_str(&format!("{s}"));
        }
    } else {
        out.push_str("00");
    }
    out.replace(':', "-")
}

/// %khz (sub_c91bb8 at 0xc91bb8): the DSD rates print as DSD64/DSD128/
/// DSD256; everything else is the sample rate divided by 1000, shown
/// without decimals when it divides evenly ("%.0f") and with one decimal
/// otherwise ("%.1f").
fn mask_sample_rate(sr: u32) -> String {
    match sr {
        2_822_400 => "DSD64".to_string(),
        5_644_800 => "DSD128".to_string(),
        11_289_600 => "DSD256".to_string(),
        _ => {
            let khz = f64::from(sr) / 1000.0;
            if sr % 1000 == 0 {
                format!("{khz:.0}")
            } else {
                format!("{khz:.1}")
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct RenameOptions {
    pub case: Case,
    pub case_enabled: bool,
    pub replace_spaces_with_underscores: bool,
    pub remove_leading_zero_in_disc: bool,
}

impl Default for RenameOptions {
    fn default() -> Self {
        RenameOptions {
            case: Case::Off,
            case_enabled: false,
            replace_spaces_with_underscores: false,
            remove_leading_zero_in_disc: false,
        }
    }
}

/// Build the new file name (stem, without extension) for one file.
pub fn build_new_name(
    file: &MusicFile,
    mask: &str,
    file_number: u32,
    file_total: u32,
    opts: &RenameOptions,
) -> Option<String> {
    let full = build_new_file_name(file, mask, file_number, file_total, opts)?;
    let ext_len = match file.extension().chars().count() {
        0 => 0,
        n => n + 1,
    };
    Some(full.chars().take(full.chars().count().saturating_sub(ext_len)).collect())
}

/// Build the full new file name, extension included, for one file.
/// Returns None when nothing could be built (the original returns the
/// resourcestring "Error in file or file does not contain tag or it new
/// name don't contain any text." when the built name trims to empty,
/// sub_ca0821 at 0xca08f5).
pub fn build_new_file_name(
    file: &MusicFile,
    mask: &str,
    file_number: u32,
    file_total: u32,
    opts: &RenameOptions,
) -> Option<String> {
    let parts = parse_mask(mask);
    let mut out = String::new();
    for part in &parts {
        match part {
            Part::Literal(l) => out.push_str(l),
            Part::Var(token) => match token {
                Token::FileName => out.push_str(&file.file_stem()),
                Token::FileNumber => out.push_str(&pad_number(file_number)),
                Token::PlayTime => out.push_str(&mask_play_time(file.duration_secs)),
                Token::Bitrate => {
                    // sub_ca0821: "VBR" (0xca15e8) goes in front of the
                    // number when the file's flag at +0x34 is set.
                    if file.is_vbr {
                        out.push_str(&format!("VBR{}", file.bitrate_kbps));
                    } else {
                        out.push_str(&file.bitrate_kbps.to_string());
                    }
                }
                Token::SampleRate => out.push_str(&mask_sample_rate(file.sample_rate)),
                // sub_c9f784 at 0xca031d: IntToStr of the item's bit-depth
                // field (+0x4c) with no test, so a file without one gives "0".
                Token::BitDepth => out.push_str(&file.bit_depth.unwrap_or(0).to_string()),
                Token::Encoder => out.push_str(&scrub_value(&file.fields.encoded_by)),
                Token::FileExt => out.push_str(&file.extension()),
                Token::Random => out.push_str(&random_number(file_total)),
                Token::Skip => {}
                Token::Tag(field) => {
                    let mut value = scrub_value(file.fields.get(*field));
                    // sub_c9f784 at 0xca0386/0xca0443: %6 and %# drop the
                    // "/total" part (Copy up to the first "/").
                    if *field == Field::Track || *field == Field::Disc {
                        if let Some(p) = value.find('/') {
                            value.truncate(p);
                        }
                        // Fresh decompile ev_57bf77ec… (sub_c9f784, lines
                        // 671-687 for %6 and 721-737 for %#): a
                        // single-character value with
                        // StrToIntDef(s, 0x7fffffff) != 0x7fffffff gets one
                        // "0" prepended (sub_40b0b4, "0" at 0xca15c4) —
                        // track 5 becomes "05" in the name.
                        if value.chars().count() == 1
                            && value.parse::<u32>().is_ok()
                        {
                            value = format!("0{value}");
                        }
                    }
                    // sub_c9f784 at 0xca04e6-0xca0502: with the option on,
                    // leading '0's become ' ' (stop at the first non-'0'),
                    // then the string is trimmed — net effect: the pad (and
                    // any other leading zeros) are dropped from %#.
                    if *field == Field::Disc && opts.remove_leading_zero_in_disc {
                        value = value.trim_start_matches('0').trim().to_string();
                    }
                    // Fresh decompile ev_57bf77ec…, sub_c9f784 lines
                    // 896-955: %9 is the comment's first line only,
                    // truncated with SetLength(s, 0x46) = 70 characters.
                    if *field == Field::Comment {
                        value = value
                            .lines()
                            .next()
                            .unwrap_or("")
                            .chars()
                            .take(70)
                            .collect();
                    }
                    out.push_str(&value);
                }
            },
        }
    }
    if out.trim().is_empty() {
        return None;
    }
    // sub_ca0821 joins the real extension onto the name before the case
    // and space steps, so Lower/UPPER/Capitalize also change ".MP3" to ".mp3".
    let ext = file.extension();
    let mut result = with_extension(out, &ext);
    if opts.case_enabled {
        result = opts.case.apply(&result);
    }
    if opts.replace_spaces_with_underscores {
        result = result.replace(' ', "_");
    }
    // sub_ca0821 at 0xca0821 (Ghidra, Nov 23 pass 4): the test runs on the
    // whole name with its extension (length over 0x104 = 260), and it is
    // skipped when the name starts with '\' (Pos("\") = 1). The kept stem
    // is cut so that stem + ".ext" is 259 (0x103) characters, then the
    // extension goes back on in with_extension. The folder part of the path
    // is not counted here, because the build does not carry the Windows path.
    let ext_len = match ext.chars().count() {
        0 => 0,
        n => n + 1,
    };
    if result.chars().count() > 260 && !result.starts_with('\\') {
        let total = result.chars().count();
        let stem_len = total.saturating_sub(ext_len);
        let keep = 259usize.saturating_sub(ext_len).min(stem_len);
        let stem: String = result.chars().take(keep).collect();
        let tail: String = result.chars().skip(stem_len).collect();
        result = stem + &tail;
    }
    // sub_ca0821 (0xca0821, the two loops after the cut): "space backslash"
    // and "backslash space" both become a bare backslash.
    while result.contains(" \\") {
        result = result.replace(" \\", "\\");
    }
    while result.contains("\\ ") {
        result = result.replace("\\ ", "\\");
    }
    Some(result)
}

/// A tag value on its way into a file name (sub_c9f73c at 0xc9f73c):
/// '\' becomes "_", and the characters a Windows file name may not
/// contain (the original's set "*?|:/<\">" at 0x6d98d0) are dropped.
fn scrub_value(value: &str) -> String {
    value
        .replace('\\', "_")
        .chars()
        .filter(|c| !"*?|:/<>\"".contains(*c))
        .collect()
}

/// The file name as it will look: the new stem with the file's own
/// extension put back (the original keeps ".mp3" after a rename).
pub fn with_extension(stem: String, ext: &str) -> String {
    if ext.is_empty() {
        stem
    } else {
        format!("{stem}.{ext}")
    }
}

/// The mask written out with the captured values in place of its
/// variables, e.g. "%1 - %2" with Artist and Title gives "Pink Floyd - Time".
pub fn render_parsed(mask: &str, pairs: &[(Field, String)]) -> String {
    let mut out = String::new();
    for part in parse_mask(mask) {
        match part {
            Part::Literal(l) => out.push_str(&l),
            Part::Var(Token::Tag(field)) => {
                if let Some((_, v)) = pairs.iter().find(|(f, _)| *f == field) {
                    out.push_str(v);
                }
            }
            Part::Var(_) => {}
        }
    }
    out
}

/// A sample name for an empty library: each variable shown as its name,
/// e.g. "<Artist> - <Title>.mp3".
pub fn placeholder_name(mask: &str) -> String {
    let mut out = String::new();
    for part in parse_mask(mask) {
        match part {
            Part::Literal(l) => out.push_str(&l),
            Part::Var(Token::Tag(field)) => out.push_str(&format!("<{}>", field.label())),
            Part::Var(Token::FileName) => out.push_str("<File name>"),
            Part::Var(Token::FileNumber) => out.push_str("<File number>"),
            Part::Var(Token::PlayTime) => out.push_str("<Play time>"),
            Part::Var(Token::Bitrate) => out.push_str("<Bitrate>"),
            Part::Var(Token::SampleRate) => out.push_str("<Hz>"),
            Part::Var(Token::BitDepth) => out.push_str("<Bit depth>"),
            Part::Var(Token::Encoder) => out.push_str("<Encoder>"),
            Part::Var(Token::FileExt) => out.push_str("<File ext.>"),
            Part::Var(Token::Random) => out.push_str("<Random>"),
            Part::Var(Token::Skip) => out.push_str("<Skip>"),
        }
    }
    with_extension(out, "mp3")
}

#[derive(Clone, Copy)]
pub struct ParseOptions {
    pub case: Case,
    pub case_enabled: bool,
    pub replace_underscores_with_spaces: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        ParseOptions {
            case: Case::Off,
            case_enabled: false,
            replace_underscores_with_spaces: false,
        }
    }
}

/// Parse a file name with a mask, returning the captured fields.
/// The extension must already be removed by the caller.
/// Mirrors the original: literals must appear in order; every variable
/// captures the text between the surrounding literals; %d discards.
pub fn parse_file_name(
    file_stem: &str,
    mask: &str,
    opts: &ParseOptions,
) -> Result<Vec<(Field, String)>, String> {
    let parts = parse_mask(mask);
    let mut out: Vec<(Field, String)> = Vec::new();
    let mut any_var = false;

    // Walk the mask: every variable captures the text up to the next
    // literal (or the end of the name); the literal itself is skipped.
    let mut rest = file_stem.to_string();
    let mut pending: Option<Token> = None;
    for part in &parts {
        match part {
            Part::Literal(l) => {
                let captured = match rest.find(l.as_str()) {
                    Some(p) => {
                        let cap = rest[..p].to_string();
                        rest = rest[p + l.len()..].to_string();
                        cap
                    }
                    None => String::new(),
                };
                if let Some(token) = pending.take() {
                    if let Some((field, value)) = capture(token, captured, opts) {
                        out.push((field, value));
                    }
                }
            }
            Part::Var(token) => {
                any_var = true;
                if let Some(prev) = pending.take() {
                    // Two variables in a row: the first captures nothing
                    // (the tab help suggests %d when a part must be split).
                    if let Some((field, value)) = capture(prev, String::new(), opts) {
                        out.push((field, value));
                    }
                }
                pending = Some(*token);
            }
        }
    }
    if let Some(token) = pending.take() {
        if let Some((field, value)) = capture(token, rest.clone(), opts) {
            out.push((field, value));
        }
    }

    if !any_var {
        return Err("The mask does not contain any variables.".into());
    }
    if out.is_empty() {
        return Err("Nothing found in the file name with this mask.".into());
    }
    Ok(out)
}

fn capture(token: Token, text: String, opts: &ParseOptions) -> Option<(Field, String)> {
    let mut value = if opts.replace_underscores_with_spaces {
        text.replace('_', " ")
    } else {
        text
    };
    if opts.case_enabled {
        value = opts.case.apply(&value);
    }
    match token {
        Token::Tag(field) => Some((field, value)),
        Token::Skip => None,
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_number_padding_matches_sub_c9f784() {
        // 0xca05ab: only numbers below 10 get one leading "0".
        assert_eq!(pad_number(1), "01");
        assert_eq!(pad_number(9), "09");
        assert_eq!(pad_number(10), "10");
        assert_eq!(pad_number(123), "123");
    }

    #[test]
    fn random_number_ranges_and_padding() {
        // 0xca05ef: <= 99 files -> Random(98)+1 padded to 2; else
        // Random(998)+1 padded to 3.
        for _ in 0..200 {
            let s = random_number(50);
            assert_eq!(s.len(), 2);
            let n: u32 = s.parse().unwrap();
            assert!((1..=98).contains(&n));
        }
        for _ in 0..200 {
            let s = random_number(100);
            assert_eq!(s.len(), 3);
            let n: u32 = s.parse().unwrap();
            assert!((1..=998).contains(&n));
        }
    }

    #[test]
    fn play_time_format_matches_sub_640954() {
        assert_eq!(mask_play_time(225.0), "03-45"); // 3:45
        assert_eq!(mask_play_time(4530.0), "01-15-30"); // 1:15:30
        assert_eq!(mask_play_time(3600.0), "01-00-00");
        assert_eq!(mask_play_time(600.0), "10-00"); // h >= 10 stays two digit
    }

    #[test]
    fn sample_rate_matches_sub_c91bb8() {
        assert_eq!(mask_sample_rate(2822400), "DSD64");
        assert_eq!(mask_sample_rate(5644800), "DSD128");
        assert_eq!(mask_sample_rate(11289600), "DSD256");
        // Whole kHz print "%.0f", fractional keep one decimal ("%.1f");
        // the divisor constant at 0xc91cf0 is 1000.0.
        assert_eq!(mask_sample_rate(48000), "48");
        assert_eq!(mask_sample_rate(8000), "8");
        assert_eq!(mask_sample_rate(44100), "44.1");
        assert_eq!(mask_sample_rate(44156), "44.2");
    }

    #[test]
    fn case_options_match_the_decompiled_processors() {
        // sub_6da398: Capitalize First Letter is title case.
        assert_eq!(
            Case::CapitalizeFirstLetter.apply("cold SIGNAL now"),
            "Cold Signal Now"
        );
        // The separator table's default (Config `CapitalizeConfiguration`,
        // literal 0xc98e84) includes digits, quotes, braces and slashes:
        // "track 2 name" capitalizes after the digit, and the separator
        // itself stays in the text.
        assert_eq!(
            Case::CapitalizeFirstLetter.apply("track 2 name (dj_set)"),
            "Track 2 Name (Dj_Set)"
        );
        // sub_6da4d8: Capitalize first word touches only the first char.
        assert_eq!(
            Case::CapitalizeFirstWord.apply("cold SIGNAL now"),
            "Cold signal now"
        );
        assert_eq!(Case::Lower.apply("COLD"), "cold");
        assert_eq!(Case::Upper.apply("cold"), "COLD");
    }

    #[test]
    fn case_option_also_changes_the_extension() {
        // sub_ca0821: the extension is joined onto the name before the case
        // step, so Lower turns ".MP3" into ".mp3".
        let mut f = crate::data::sample_file();
        f.name = "Song.MP3".to_string();
        let opts = RenameOptions {
            case: Case::Lower,
            case_enabled: true,
            ..RenameOptions::default()
        };
        assert_eq!(build_new_file_name(&f, "%7", 1, 10, &opts).unwrap(), "song.mp3");
        assert_eq!(build_new_name(&f, "%7", 1, 10, &opts).unwrap(), "song");
    }

    #[test]
    fn bit_depth_without_a_value_is_zero() {
        // sub_c9f784 at 0xca031d: IntToStr(+0x4c) with no test.
        let mut f = crate::data::sample_file();
        f.bit_depth = None;
        let opts = RenameOptions::default();
        assert_eq!(build_new_name(&f, "%dbit", 1, 10, &opts).unwrap(), "0");
        f.bit_depth = Some(24);
        assert_eq!(build_new_name(&f, "%dbit", 1, 10, &opts).unwrap(), "24");
    }

    #[test]
    fn values_are_scrubbed_like_the_original() {
        // sub_c9f73c: '\' -> '_'; sub_6d9834's set is dropped.
        assert_eq!(scrub_value("a\\b"), "a_b");
        assert_eq!(scrub_value("a:b*?"), "ab");
    }

    #[test]
    fn long_names_are_cut_to_259_with_the_extension_counted() {
        // sub_ca0821 (0xca0821): over 260 with the extension is cut so that
        // stem + ".ext" is 259; a name starting with '\' is not cut.
        let f = crate::data::sample_file();
        let opts = RenameOptions::default();
        let long = "x".repeat(300);
        let stem = build_new_name(&f, &long, 1, 10, &opts).unwrap();
        let ext_len = f.extension().chars().count() + 1;
        assert_eq!(stem.chars().count(), 259 - ext_len);
        let kept = build_new_name(&f, &"x".repeat(250), 1, 10, &opts).unwrap();
        assert_eq!(kept.chars().count(), 250);
        // sub_ca0821's last loops (0xca0821 lines 150-159): a space next to
        // a backslash is removed, so the folder separator is clean.
        assert_eq!(build_new_name(&f, "%2 \\ %1", 1, 10, &opts).unwrap(), "Northern Lights\\Aurora Skyline");
        let folder = format!("\\{}", "x".repeat(300));
        assert_eq!(build_new_name(&f, &folder, 1, 10, &opts).unwrap().chars().count(), 301);
    }

    #[test]
    fn rename_builds_a_name_from_tokens() {
        let f = crate::data::sample_file();
        let opts = RenameOptions::default();
        let name = build_new_name(&f, "%1 - %2", 3, 10, &opts).unwrap();
        assert_eq!(name, "Aurora Skyline - Northern Lights");
        // %8 below ten is padded, above ten is not.
        assert_eq!(build_new_name(&f, "%8", 3, 10, &opts).unwrap(), "03");
        assert_eq!(build_new_name(&f, "%8", 12, 10, &opts).unwrap(), "12");
    }

    #[test]
    fn podcast_tokens_use_their_own_fields() {
        // sub_c9f784 reads %swt/%eid/%snr/%enr from the item's podcast/TV
        // fields (+0x12c..+0x138), not from Lyricist/Catalog/Isrc.
        let mut f = crate::data::sample_file();
        f.fields.show_title = "Show".into();
        f.fields.episode_id = "EID".into();
        f.fields.season_number = "2".into();
        f.fields.episode_number = "7".into();
        let opts = RenameOptions::default();
        assert_eq!(build_new_name(&f, "%swt", 1, 10, &opts).unwrap(), "Show");
        assert_eq!(build_new_name(&f, "%eid", 1, 10, &opts).unwrap(), "EID");
        assert_eq!(build_new_name(&f, "%snr", 1, 10, &opts).unwrap(), "2");
        assert_eq!(build_new_name(&f, "%enr", 1, 10, &opts).unwrap(), "7");
    }

    #[test]
    fn track_and_disc_pad_single_digits_like_sub_c9f784() {
        // 0xca03d0: after the "/total" drop, a single-character value that
        // parses as an integer gets "0" prepended — track 5 is "05".
        let mut f = crate::data::sample_file();
        f.fields.track = "5".into();
        f.fields.track_total = "12".into();
        f.fields.disc = "5".into();
        let opts = RenameOptions::default();
        assert_eq!(build_new_name(&f, "%6", 1, 10, &opts).unwrap(), "05");
        assert_eq!(build_new_name(&f, "%#", 1, 10, &opts).unwrap(), "05");
        // Two-digit values are untouched, non-numeric single chars are not
        // padded (StrToIntDef returns the MaxInt default).
        f.fields.track = "45".into();
        assert_eq!(build_new_name(&f, "%6", 1, 10, &opts).unwrap(), "45");
        f.fields.track = "x".into();
        assert_eq!(build_new_name(&f, "%6", 1, 10, &opts).unwrap(), "x");
        // The disc option (form+0xc89) strips the pad again: "05" -> "5".
        let opts_strip = RenameOptions {
            remove_leading_zero_in_disc: true,
            ..Default::default()
        };
        f.fields.disc = "05".into();
        assert_eq!(build_new_name(&f, "%#", 1, 10, &opts_strip).unwrap(), "5");
    }
}
