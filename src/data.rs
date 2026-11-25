//! The music-file model: one row per audio file, its tag fields, its
//! audio properties, and reading/writing through lofty.

use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use lofty::config::WriteOptions;
use lofty::file::{AudioFile, FileType, TaggedFile, TaggedFileExt};
use lofty::picture::{MimeType, Picture, PictureType};
use lofty::prelude::Accessor;
use lofty::probe::Probe;
use lofty::tag::items::Timestamp;
use lofty::config::ParseOptions;
use lofty::id3::v2::{Frame, FrameId, TextInformationFrame};
use lofty::mpeg::MpegFile;
use lofty::tag::{ItemKey, Tag};
use lofty::TextEncoding;
use std::borrow::Cow;

/// The extensions the original handles (Readme.txt): mp3, mpc/mp+,
/// wma/asf/wmv, ogg/oga/opus, flac, aac/m4a/mp4, ape, wv, ofr, wav, aiff.
pub const AUDIO_EXTENSIONS: &[&str] = &[
    "mp3", "mp2", "mpc", "mp+", "wma", "asf", "wmv", "ogg", "oga", "opus",
    "spx", "flac", "aac", "m4a", "m4b", "m4p", "mp4", "ape", "wv", "ofr",
    "ofs", "wav", "aif", "aiff", "aifc",
];

pub fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| AUDIO_EXTENSIONS.iter().any(|a| a.eq_ignore_ascii_case(e)))
        .unwrap_or(false)
}

/// Tag field identifiers, mirroring the original's field set.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Field {
    Title,
    Artist,
    Album,
    AlbumArtist,
    Year,
    Genre,
    Comment,
    Composer,
    OrigArtist,
    Copyright,
    Url,
    EncodedBy,
    Track,
    TrackTotal,
    Disc,
    DiscTotal,
    Bpm,
    Mood,
    Rating,
    Lyrics,
    Conductor,
    Lyricist,
    Label,
    RemixedBy,
    Isrc,
    Catalog,
    InitialKey,
    // The original's Additional tab: original lyricist, radio name and
    // the five webpage fields.
    OrigLyricist,
    RadioName,
    FilePage,
    ArtistPage,
    SourcePage,
    RadioPage,
    BuyPage,
    // The original's "More Additional" (iTunes/podcast) and Sorting tabs.
    Grouping,
    Subtitle,
    ReleaseTime,
    PodcastCategory,
    PodcastDescription,
    PodcastId,
    PodcastUrl,
    ShowTitle,
    EpisodeId,
    SeasonNumber,
    EpisodeNumber,
    SortTitle,
    SortArtist,
    SortAlbumArtist,
    SortAlbum,
    SortComposer,
    SortShow,
    // MusicBrainz identifiers and release details (the MusicBrainz tab).
    MbAlbumType,
    MbAlbumArtistId,
    MbAlbumId,
    MbAlbumStatus,
    MbArtistId,
    MbRecordingId,
    MbReleaseGroupId,
    MbReleaseTrackId,
    MbReleaseType,
    MbTrackId,
    MbAlbumReleaseCountry,
    MbWorkId,
    // ReplayGain gain and peak values (the ReplayGain tab).
    RgTrackGain,
    RgTrackPeak,
    RgAlbumGain,
    RgAlbumPeak,
    Compilation,
}

impl Field {
    pub fn label(self) -> &'static str {
        match self {
            Field::Title => "Title",
            Field::Artist => "Artist",
            Field::Album => "Album",
            Field::AlbumArtist => "Album Artist",
            Field::Year => "Year",
            Field::Genre => "Genre",
            Field::Comment => "Comment",
            Field::Composer => "Composer",
            Field::OrigArtist => "Orig. Artist",
            Field::Copyright => "Copyright",
            Field::Url => "URL",
            Field::EncodedBy => "Encoded by",
            Field::Track => "Track #",
            Field::TrackTotal => "Track total",
            Field::Disc => "Disc #",
            Field::DiscTotal => "Disc total",
            Field::Bpm => "BPM",
            Field::Mood => "Mood",
            Field::Rating => "Rating",
            Field::Lyrics => "Lyrics",
            Field::Conductor => "Conductor",
            Field::Lyricist => "Lyricist",
            Field::Label => "Label",
            Field::RemixedBy => "Remixed by",
            Field::Isrc => "ISRC #",
            Field::Catalog => "Catalog #",
            Field::InitialKey => "Initial Key",
            Field::OrigLyricist => "Orig. Lyricist",
            Field::RadioName => "Radio station name",
            Field::FilePage => "File information webpage",
            Field::ArtistPage => "Artist webpage",
            Field::SourcePage => "Audio source webpage",
            Field::RadioPage => "Radio station webpage",
            Field::BuyPage => "Buy CD webpage",
            Field::Grouping => "Grouping",
            Field::Subtitle => "Subtitle",
            Field::ReleaseTime => "Release Time",
            Field::PodcastCategory => "Podcast Category",
            Field::PodcastDescription => "Podcast Description",
            Field::PodcastId => "Podcast ID",
            Field::PodcastUrl => "Podcast URL",
            Field::ShowTitle => "Show Title",
            Field::EpisodeId => "Episode ID",
            Field::SeasonNumber => "Season Number",
            Field::EpisodeNumber => "Episode Number",
            Field::SortTitle => "Sort Title",
            Field::SortArtist => "Sort Artist",
            Field::SortAlbumArtist => "Sort Album Artist",
            Field::SortAlbum => "Sort Album",
            Field::SortComposer => "Sort Composer",
            Field::SortShow => "Sort Show",
            Field::MbAlbumType => "musicbrainz album type",
            Field::MbAlbumArtistId => "musicbrainz album artist id",
            Field::MbAlbumId => "musicbrainz album id",
            Field::MbAlbumStatus => "musicbrainz album status",
            Field::MbArtistId => "musicbrainz artist id",
            Field::MbRecordingId => "musicbrainz recording id",
            Field::MbReleaseGroupId => "musicbrainz release group id",
            Field::MbReleaseTrackId => "musicbrainz release track id",
            Field::MbReleaseType => "musicbrainz release type",
            Field::MbTrackId => "musicbrainz track id",
            Field::MbAlbumReleaseCountry => "musicbrainz album release country",
            Field::MbWorkId => "musicbrainz work id",
            Field::RgTrackGain => "Track gain",
            Field::RgTrackPeak => "Track peak",
            Field::RgAlbumGain => "Album gain",
            Field::RgAlbumPeak => "Album peak",
            Field::Compilation => "Part of a compilation",
        }
    }

}

/// Every editable tag field, in the original's grouping order.
pub const ALL_FIELDS: &[Field] = &[
    Field::Title,
    Field::Artist,
    Field::Album,
    Field::AlbumArtist,
    Field::Year,
    Field::Genre,
    Field::Comment,
    Field::Track,
    Field::TrackTotal,
    Field::Disc,
    Field::DiscTotal,
    Field::Bpm,
    Field::Mood,
    Field::Rating,
    Field::Compilation,
    Field::Composer,
    Field::OrigArtist,
    Field::Copyright,
    Field::Url,
    Field::EncodedBy,
    Field::Lyrics,
    Field::Conductor,
    Field::Lyricist,
    Field::Label,
    Field::RemixedBy,
    Field::Isrc,
    Field::Catalog,
    Field::InitialKey,
    Field::OrigLyricist,
    Field::RadioName,
    Field::FilePage,
    Field::ArtistPage,
    Field::SourcePage,
    Field::RadioPage,
    Field::BuyPage,
    Field::Grouping,
    Field::Subtitle,
    Field::ReleaseTime,
    Field::PodcastCategory,
    Field::PodcastDescription,
    Field::PodcastId,
    Field::PodcastUrl,
    Field::ShowTitle,
    Field::EpisodeId,
    Field::SeasonNumber,
    Field::EpisodeNumber,
    Field::SortTitle,
    Field::SortArtist,
    Field::SortAlbumArtist,
    Field::SortAlbum,
    Field::SortComposer,
    Field::SortShow,
    Field::MbAlbumType,
    Field::MbAlbumArtistId,
    Field::MbAlbumId,
    Field::MbAlbumStatus,
    Field::MbArtistId,
    Field::MbRecordingId,
    Field::MbReleaseGroupId,
    Field::MbReleaseTrackId,
    Field::MbReleaseType,
    Field::MbTrackId,
    Field::MbAlbumReleaseCountry,
    Field::MbWorkId,
    Field::RgTrackGain,
    Field::RgTrackPeak,
    Field::RgAlbumGain,
    Field::RgAlbumPeak,
];

/// The fields offered by the Export window.
pub const EXPORT_FIELDS: &[Field] = &[
    Field::Artist,
    Field::Title,
    Field::Album,
    Field::AlbumArtist,
    Field::Year,
    Field::Track,
    Field::TrackTotal,
    Field::Disc,
    Field::Genre,
    Field::Bpm,
    Field::Mood,
    Field::Rating,
    Field::Comment,
    Field::Composer,
    Field::Conductor,
    Field::OrigArtist,
    Field::Copyright,
    Field::Url,
    Field::EncodedBy,
    Field::Label,
    Field::RemixedBy,
    Field::Isrc,
    Field::Catalog,
    Field::InitialKey,
];

#[derive(Clone, Debug, Default)]
pub struct TagFields {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: String,
    pub year: String,
    pub genre: String,
    pub comment: String,
    pub track: String,
    pub track_total: String,
    pub disc: String,
    pub disc_total: String,
    pub bpm: String,
    pub mood: String,
    pub rating: String,
    pub lyrics: String,
    pub composer: String,
    pub orig_artist: String,
    pub copyright: String,
    pub url: String,
    pub encoded_by: String,
    pub conductor: String,
    pub lyricist: String,
    pub label: String,
    pub remixed_by: String,
    pub isrc: String,
    pub catalog: String,
    pub initial_key: String,
    pub orig_lyricist: String,
    pub radio_name: String,
    pub file_page: String,
    pub artist_page: String,
    pub source_page: String,
    pub radio_page: String,
    pub buy_page: String,
    pub grouping: String,
    pub subtitle: String,
    pub release_time: String,
    pub podcast_category: String,
    pub podcast_description: String,
    pub podcast_id: String,
    pub podcast_url: String,
    pub show_title: String,
    pub episode_id: String,
    pub season_number: String,
    pub episode_number: String,
    pub sort_title: String,
    pub sort_artist: String,
    pub sort_album_artist: String,
    pub sort_album: String,
    pub sort_composer: String,
    pub sort_show: String,
    pub mb_album_type: String,
    pub mb_album_artist_id: String,
    pub mb_album_id: String,
    pub mb_album_status: String,
    pub mb_artist_id: String,
    pub mb_recording_id: String,
    pub mb_release_group_id: String,
    pub mb_release_track_id: String,
    pub mb_release_type: String,
    pub mb_track_id: String,
    pub mb_album_release_country: String,
    pub mb_work_id: String,
    pub rg_track_gain: String,
    pub rg_track_peak: String,
    pub rg_album_gain: String,
    pub rg_album_peak: String,
    pub compilation: Option<bool>,
}

impl TagFields {
    pub fn get(&self, f: Field) -> &str {
        match f {
            Field::Title => &self.title,
            Field::Artist => &self.artist,
            Field::Album => &self.album,
            Field::AlbumArtist => &self.album_artist,
            Field::Year => &self.year,
            Field::Genre => &self.genre,
            Field::Comment => &self.comment,
            Field::Track => &self.track,
            Field::TrackTotal => &self.track_total,
            Field::Disc => &self.disc,
            Field::DiscTotal => &self.disc_total,
            Field::Bpm => &self.bpm,
            Field::Mood => &self.mood,
            Field::Rating => &self.rating,
            Field::Lyrics => &self.lyrics,
            Field::Composer => &self.composer,
            Field::OrigArtist => &self.orig_artist,
            Field::Copyright => &self.copyright,
            Field::Url => &self.url,
            Field::EncodedBy => &self.encoded_by,
            Field::Conductor => &self.conductor,
            Field::Lyricist => &self.lyricist,
            Field::Label => &self.label,
            Field::RemixedBy => &self.remixed_by,
            Field::Isrc => &self.isrc,
            Field::Catalog => &self.catalog,
            Field::InitialKey => &self.initial_key,
            Field::OrigLyricist => &self.orig_lyricist,
            Field::RadioName => &self.radio_name,
            Field::FilePage => &self.file_page,
            Field::ArtistPage => &self.artist_page,
            Field::SourcePage => &self.source_page,
            Field::RadioPage => &self.radio_page,
            Field::BuyPage => &self.buy_page,
            Field::Grouping => &self.grouping,
            Field::Subtitle => &self.subtitle,
            Field::ReleaseTime => &self.release_time,
            Field::PodcastCategory => &self.podcast_category,
            Field::PodcastDescription => &self.podcast_description,
            Field::PodcastId => &self.podcast_id,
            Field::PodcastUrl => &self.podcast_url,
            Field::ShowTitle => &self.show_title,
            Field::EpisodeId => &self.episode_id,
            Field::SeasonNumber => &self.season_number,
            Field::EpisodeNumber => &self.episode_number,
            Field::SortTitle => &self.sort_title,
            Field::SortArtist => &self.sort_artist,
            Field::SortAlbumArtist => &self.sort_album_artist,
            Field::SortAlbum => &self.sort_album,
            Field::SortComposer => &self.sort_composer,
            Field::SortShow => &self.sort_show,
            Field::MbAlbumType => &self.mb_album_type,
            Field::MbAlbumArtistId => &self.mb_album_artist_id,
            Field::MbAlbumId => &self.mb_album_id,
            Field::MbAlbumStatus => &self.mb_album_status,
            Field::MbArtistId => &self.mb_artist_id,
            Field::MbRecordingId => &self.mb_recording_id,
            Field::MbReleaseGroupId => &self.mb_release_group_id,
            Field::MbReleaseTrackId => &self.mb_release_track_id,
            Field::MbReleaseType => &self.mb_release_type,
            Field::MbTrackId => &self.mb_track_id,
            Field::MbAlbumReleaseCountry => &self.mb_album_release_country,
            Field::MbWorkId => &self.mb_work_id,
            Field::RgTrackGain => &self.rg_track_gain,
            Field::RgTrackPeak => &self.rg_track_peak,
            Field::RgAlbumGain => &self.rg_album_gain,
            Field::RgAlbumPeak => &self.rg_album_peak,
            Field::Compilation => "",
        }
    }

    pub fn set(&mut self, f: Field, value: &str) {
        let dst: &mut String = match f {
            Field::Title => &mut self.title,
            Field::Artist => &mut self.artist,
            Field::Album => &mut self.album,
            Field::AlbumArtist => &mut self.album_artist,
            Field::Year => &mut self.year,
            Field::Genre => &mut self.genre,
            Field::Comment => &mut self.comment,
            Field::Track => &mut self.track,
            Field::TrackTotal => &mut self.track_total,
            Field::Disc => &mut self.disc,
            Field::DiscTotal => &mut self.disc_total,
            Field::Bpm => &mut self.bpm,
            Field::Mood => &mut self.mood,
            Field::Rating => &mut self.rating,
            Field::Lyrics => &mut self.lyrics,
            Field::Composer => &mut self.composer,
            Field::OrigArtist => &mut self.orig_artist,
            Field::Copyright => &mut self.copyright,
            Field::Url => &mut self.url,
            Field::EncodedBy => &mut self.encoded_by,
            Field::Conductor => &mut self.conductor,
            Field::Lyricist => &mut self.lyricist,
            Field::Label => &mut self.label,
            Field::RemixedBy => &mut self.remixed_by,
            Field::Isrc => &mut self.isrc,
            Field::Catalog => &mut self.catalog,
            Field::InitialKey => &mut self.initial_key,
            Field::OrigLyricist => &mut self.orig_lyricist,
            Field::RadioName => &mut self.radio_name,
            Field::FilePage => &mut self.file_page,
            Field::ArtistPage => &mut self.artist_page,
            Field::SourcePage => &mut self.source_page,
            Field::RadioPage => &mut self.radio_page,
            Field::BuyPage => &mut self.buy_page,
            Field::Grouping => &mut self.grouping,
            Field::Subtitle => &mut self.subtitle,
            Field::ReleaseTime => &mut self.release_time,
            Field::PodcastCategory => &mut self.podcast_category,
            Field::PodcastDescription => &mut self.podcast_description,
            Field::PodcastId => &mut self.podcast_id,
            Field::PodcastUrl => &mut self.podcast_url,
            Field::ShowTitle => &mut self.show_title,
            Field::EpisodeId => &mut self.episode_id,
            Field::SeasonNumber => &mut self.season_number,
            Field::EpisodeNumber => &mut self.episode_number,
            Field::SortTitle => &mut self.sort_title,
            Field::SortArtist => &mut self.sort_artist,
            Field::SortAlbumArtist => &mut self.sort_album_artist,
            Field::SortAlbum => &mut self.sort_album,
            Field::SortComposer => &mut self.sort_composer,
            Field::SortShow => &mut self.sort_show,
            Field::MbAlbumType => &mut self.mb_album_type,
            Field::MbAlbumArtistId => &mut self.mb_album_artist_id,
            Field::MbAlbumId => &mut self.mb_album_id,
            Field::MbAlbumStatus => &mut self.mb_album_status,
            Field::MbArtistId => &mut self.mb_artist_id,
            Field::MbRecordingId => &mut self.mb_recording_id,
            Field::MbReleaseGroupId => &mut self.mb_release_group_id,
            Field::MbReleaseTrackId => &mut self.mb_release_track_id,
            Field::MbReleaseType => &mut self.mb_release_type,
            Field::MbTrackId => &mut self.mb_track_id,
            Field::MbAlbumReleaseCountry => &mut self.mb_album_release_country,
            Field::MbWorkId => &mut self.mb_work_id,
            Field::RgTrackGain => &mut self.rg_track_gain,
            Field::RgTrackPeak => &mut self.rg_track_peak,
            Field::RgAlbumGain => &mut self.rg_album_gain,
            Field::RgAlbumPeak => &mut self.rg_album_peak,
            Field::Compilation => return,
        };
        *dst = value.to_string();
    }
}

/// One row of the file list.
#[derive(Clone, Debug)]
pub struct MusicFile {
    pub path: PathBuf,
    /// Display name without path (with extension).
    pub name: String,
    /// True when this row represents a folder in the main list.
    pub is_dir: bool,
    /// Human name of the tag type, e.g. "ID3v2", "Vorbis", "" when none.
    pub tag_kind: String,
    pub fields: TagFields,
    pub duration_secs: f64,
    pub bitrate_kbps: u32,
    pub sample_rate: u32,
    pub channels: u8,
    pub bit_depth: Option<u8>,
    pub size_bytes: u64,
    /// Seconds since the epoch, for the Date Modified column (a file
    /// property of the original; the classic column subset we show does
    /// not include it).
    #[allow(dead_code)]
    pub modified_secs: u64,
    /// True when the bitrate is variable: the original prefixes %0 with
    /// "VBR" for such files (flag at item+0x34, sub_ca0821).
    pub is_vbr: bool,
    /// First embedded cover: (short format name, bytes).
    pub cover: Option<(String, std::sync::Arc<Vec<u8>>)>,
}

impl MusicFile {
    pub fn file_stem(&self) -> String {
        Path::new(&self.name)
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.name.clone())
    }

    pub fn extension(&self) -> String {
        if self.is_dir { return String::new(); }
        Path::new(&self.name)
            .extension()
            .map(|e| e.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    pub fn play_time(&self) -> String {
        let secs = self.duration_secs.round() as u64;
        if secs == 0 { return String::new(); }
        if secs >= 3600 {
            format!("{}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60)
        } else {
            format!("{}:{:02}", secs / 60, secs % 60)
        }
    }

    pub fn size_text(&self) -> String {
        if self.size_bytes == 0 { return String::new(); }
        let kb = self.size_bytes / 1024;
        if kb >= 1024 * 1024 {
            format!("{:.1} GB", self.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        } else if kb >= 1024 {
            format!("{:.1} MB", self.size_bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{kb} KB")
        }
    }

    /// "500 x 500, JPEG" style summary, like the original's picture label.
    pub fn picture_info(&self) -> Option<String> {
        self.cover.as_ref().map(|(mime, bytes)| {
            if let Ok((w, h)) = image_dims(bytes) {
                format!("{w} x {h}, {mime}")
            } else {
                mime.clone()
            }
        })
    }
}

pub fn read_folder_row(path: &Path) -> Option<MusicFile> {
    use std::collections::BTreeSet;

    let name = path.file_name()?.to_string_lossy().into_owned();
    let meta = std::fs::metadata(path).ok();
    let mut modified_secs = meta
        .as_ref()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // A folder row shows the same shared columns as a song row, filled
    // with an aggregate of the audio files directly inside it: track
    // count, artists, years, genres, total time, average bitrate and
    // total size. Reading a bounded number keeps opening a big library
    // responsive; deeper folders are aggregated when they are expanded.
    let mut count = 0usize;
    let mut duration = 0.0f64;
    let mut size = 0u64;
    let mut bitrate_sum = 0u64;
    let mut artists = BTreeSet::new();
    let mut albums = BTreeSet::new();
    let mut years = BTreeSet::new();
    let mut genres = BTreeSet::new();
    let mut kinds = BTreeSet::new();
    if let Ok(rd) = std::fs::read_dir(path) {
        for entry in rd.flatten().take(400) {
            let p = entry.path();
            if !p.is_file() || !is_audio_file(&p) {
                continue;
            }
            if let Ok(m) = std::fs::metadata(&p) {
                size += m.len();
                if let Ok(mtime) = m.modified() {
                    if let Ok(t) = mtime.duration_since(UNIX_EPOCH) {
                        modified_secs = modified_secs.max(t.as_secs());
                    }
                }
            }
            let Some(f) = read_file(&p) else { continue };
            count += 1;
            duration += f.duration_secs;
            bitrate_sum += u64::from(f.bitrate_kbps);
            if !f.fields.artist.is_empty() { artists.insert(f.fields.artist); }
            if !f.fields.album.is_empty() { albums.insert(f.fields.album); }
            if !f.fields.year.is_empty() { years.insert(f.fields.year); }
            if !f.fields.genre.is_empty() { genres.insert(f.fields.genre); }
            if !f.tag_kind.is_empty() { kinds.insert(f.tag_kind); }
        }
    }
    fn join_limited(set: &BTreeSet<String>) -> String {
        set.iter().take(2).cloned().collect::<Vec<_>>().join(", ")
            + if set.len() > 2 { "…" } else { "" }
    }
    let mut fields = TagFields::default();
    if count > 0 {
        fields.artist = join_limited(&artists);
        fields.album = join_limited(&albums);
        fields.year = join_limited(&years);
        fields.genre = join_limited(&genres);
        fields.track = count.to_string();
    }
    Some(MusicFile {
        path: path.to_path_buf(),
        name,
        is_dir: true,
        tag_kind: if kinds.is_empty() { "Folder".into() } else { join_limited(&kinds) },
        fields,
        duration_secs: duration,
        bitrate_kbps: if count == 0 { 0 } else { (bitrate_sum / count as u64) as u32 },
        sample_rate: 0,
        channels: 0,
        bit_depth: None,
        size_bytes: size,
        modified_secs,
        is_vbr: false,
        cover: None,
    })
}

/// Dimensions of an embedded picture, for the "Picture: W x H, TYPE"
/// label (the original's format `Picture: %d x %d, %s`, 0x9ece3e).
pub fn image_dims(bytes: &[u8]) -> Result<(u32, u32), String> {
    let img = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
    Ok((img.width(), img.height()))
}

fn tag_kind_name(tag_type: lofty::tag::TagType) -> String {
    use lofty::tag::TagType::*;
    match tag_type {
        Id3v2 => "ID3v2".into(),
        Id3v1 => "ID3v1".into(),
        Ape => "APEv2".into(),
        VorbisComments => "Vorbis".into(),
        Mp4Ilst => "MP4".into(),
        RiffInfo => "WAV INFO".into(),
        AiffText => "AIFF".into(),
        // lofty marks TagType non-exhaustive; anything else is just "Tag".
        _ => "Tag".into(),
    }
}

/// The original trims a full timestamp down to the year ("Trim Year frame
/// if it is contain timestamp ... instead of year").
fn trim_year(raw: &str) -> String {
    let digits: String = raw.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 4 {
        digits.chars().take(4).collect()
    } else {
        raw.to_string()
    }
}

/// Read one text frame straight from an MP3's ID3v2 tag. lofty drops some
/// frames that are legal in the wild (for example a v2.4 TDRC timestamp in
/// a v2.3 tag), so the fields that matter to the original are read here.
fn id3v2_text_frame(path: &Path, want: &[u8; 4]) -> Option<String> {
    use std::io::{Read, Seek, SeekFrom};

    fn syncsafe(b: &[u8]) -> u64 {
        b.iter().fold(0, |acc, &x| (acc << 7) | (x & 0x7f) as u64)
    }
    fn plain(b: &[u8]) -> u64 {
        b.iter().fold(0, |acc, &x| (acc << 8) | x as u64)
    }

    let mut file = std::fs::File::open(path).ok()?;
    let mut header = [0u8; 10];
    file.read_exact(&mut header).ok()?;
    if &header[0..3] != b"ID3" || !(3..=4).contains(&header[3]) {
        return None;
    }
    let version = header[3];
    let tag_end = 10 + syncsafe(&header[6..10]);
    let mut pos = 10u64;
    if header[5] & 0x40 != 0 {
        let mut size = [0u8; 4];
        file.read_exact(&mut size).ok()?;
        let ext = if version == 4 { syncsafe(&size) } else { plain(&size) + 4 };
        pos += ext;
        file.seek(SeekFrom::Start(pos)).ok()?;
    }
    while pos + 10 <= tag_end {
        let mut fh = [0u8; 10];
        if file.read_exact(&mut fh).is_err() || fh[0] == 0 {
            break;
        }
        let size = if version == 4 { syncsafe(&fh[4..8]) } else { plain(&fh[4..8]) };
        pos += 10 + size;
        if &fh[0..4] == want {
            if size == 0 || size > 1024 {
                return None;
            }
            let mut body = vec![0u8; size as usize];
            file.read_exact(&mut body).ok()?;
            return id3v2_text(&body);
        }
        file.seek(SeekFrom::Start(pos)).ok()?;
    }
    None
}

/// The track number as the MP3's ID3v2 TRCK frame holds it, leading zero
/// included ("01"). lofty reads that text as a number, so the zero is lost
/// unless the frame is read here.
fn id3v2_track_number(path: &Path) -> Option<String> {
    id3v2_text_frame(path, b"TRCK").and_then(|text| {
        let number = text.split('/').next()?.trim().trim_matches('\0').trim();
        let digits = !number.is_empty() && number.chars().all(|c| c.is_ascii_digit());
        digits.then(|| number.to_string())
    })
}

/// Year from the frame the original uses: ID3v2.4 TDRC (Recorded date), or
/// TYER on older tags. A full timestamp such as 2003-01-01 is trimmed to
/// its four-digit year.
fn id3v2_year(path: &Path) -> Option<String> {
    for frame in [b"TDRC", b"TYER"] {
        if let Some(text) = id3v2_text_frame(path, frame) {
            let year = trim_year(&text);
            if year.len() == 4 && year.chars().all(|c| c.is_ascii_digit()) {
                return Some(year);
            }
        }
    }
    None
}

/// The text of an ID3v2 text frame body: an encoding byte, then the text.
fn id3v2_text(body: &[u8]) -> Option<String> {
    let (&encoding, text) = body.split_first()?;
    let text = match encoding {
        1 | 2 => {
            let (big_endian, units) = match text {
                [0xFF, 0xFE, rest @ ..] => (false, rest),
                [0xFE, 0xFF, rest @ ..] => (true, rest),
                _ => (encoding == 2, text),
            };
            let units: Vec<u16> = units
                .chunks_exact(2)
                .map(|p| {
                    if big_endian {
                        u16::from_be_bytes([p[0], p[1]])
                    } else {
                        u16::from_le_bytes([p[0], p[1]])
                    }
                })
                .take_while(|&u| u != 0)
                .collect();
            String::from_utf16_lossy(&units)
        }
        0 => text.iter().take_while(|&&b| b != 0).map(|&b| b as char).collect(),
        _ => String::from_utf8_lossy(text.split(|&b| b == 0).next().unwrap_or(&[])).into_owned(),
    };
    Some(text)
}

/// Read one file's tags and properties with lofty.
pub fn read_file(path: &Path) -> Option<MusicFile> {
    let tagged: TaggedFile = Probe::open(path).ok()?.read().ok()?;
    let props = tagged.properties();
    let duration_secs = props.duration().as_secs_f64();
    let bitrate_kbps = props.audio_bitrate().unwrap_or(0);
    let sample_rate = props.sample_rate().unwrap_or(0);
    let channels = props.channels().unwrap_or(0);
    let bit_depth = props.bit_depth();

    let meta = std::fs::metadata(path).ok();
    let size_bytes = meta.as_ref().map(|m| m.len()).unwrap_or(0);
    let modified_secs = meta
        .as_ref()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let tag = tagged.primary_tag().cloned();

    let (tag_kind, mut fields, cover) = match &tag {
        Some(t) => {
            let kind = tag_kind_name(t.tag_type());
            let mut f = TagFields::default();
            f.title = t.title().unwrap_or_default().into_owned();
            f.artist = t.artist().unwrap_or_default().into_owned();
            f.album = t.album().unwrap_or_default().into_owned();
            f.album_artist = t
                .get_string(ItemKey::AlbumArtist)
                .unwrap_or_default()
                .to_string();
            f.year = t.date().map(|d| d.year.to_string()).unwrap_or_default();
            f.genre = t.genre().unwrap_or_default().into_owned();
            f.comment = t.comment().unwrap_or_default().into_owned();
            f.track = t
                .get_string(ItemKey::TrackNumber)
                .map(str::to_string)
                .or_else(|| t.track().map(|v| v.to_string()))
                .unwrap_or_default();
            if t.tag_type() == lofty::tag::TagType::Id3v2 {
                if let Some(raw) = id3v2_track_number(path) {
                    if raw.parse::<u32>().ok() == f.track.parse::<u32>().ok() {
                        f.track = raw;
                    }
                }
            }
            f.track_total = t
                .get_string(ItemKey::TrackTotal)
                .map(str::to_string)
                .or_else(|| t.track_total().map(|v| v.to_string()))
                .unwrap_or_default();
            f.disc = t.disk().map(|v| v.to_string()).unwrap_or_default();
            f.disc_total = t.disk_total().map(|v| v.to_string()).unwrap_or_default();
            f.composer = t.get_string(ItemKey::Composer).unwrap_or_default().into();
            f.orig_artist = t.get_string(ItemKey::OriginalArtist).unwrap_or_default().into();
            f.copyright = t.get_string(ItemKey::CopyrightMessage).unwrap_or_default().into();
            f.url = t.get_string(ItemKey::TrackArtistUrl).unwrap_or_default().into();
            f.encoded_by = t.get_string(ItemKey::EncodedBy).unwrap_or_default().into();
            f.bpm = t.get_string(ItemKey::IntegerBpm).unwrap_or_default().into();
            f.mood = t.get_string(ItemKey::Mood).unwrap_or_default().into();
            f.rating = t.get_string(ItemKey::Popularimeter).unwrap_or_default().into();
            f.lyrics = t
                .get_string(ItemKey::UnsyncLyrics)
                .or_else(|| t.get_string(ItemKey::Lyrics))
                .unwrap_or_default()
                .into();
            f.conductor = t.get_string(ItemKey::Conductor).unwrap_or_default().into();
            f.lyricist = t.get_string(ItemKey::Lyricist).unwrap_or_default().into();
            f.label = t.get_string(ItemKey::Label).unwrap_or_default().into();
            f.remixed_by = t.get_string(ItemKey::Remixer).unwrap_or_default().into();
            f.isrc = t.get_string(ItemKey::Isrc).unwrap_or_default().into();
            f.catalog = t.get_string(ItemKey::CatalogNumber).unwrap_or_default().into();
            f.initial_key = t.get_string(ItemKey::InitialKey).unwrap_or_default().into();
            f.orig_lyricist = t.get_string(ItemKey::OriginalLyricist).unwrap_or_default().into();
            f.radio_name = t.get_string(ItemKey::InternetRadioStationName).unwrap_or_default().into();
            f.file_page = t.get_string(ItemKey::AudioFileUrl).unwrap_or_default().into();
            f.artist_page = t.get_string(ItemKey::TrackArtistUrl).unwrap_or_default().into();
            f.source_page = t.get_string(ItemKey::AudioSourceUrl).unwrap_or_default().into();
            f.radio_page = t.get_string(ItemKey::RadioStationUrl).unwrap_or_default().into();
            f.buy_page = t.get_string(ItemKey::PaymentUrl).unwrap_or_default().into();
            f.grouping = t.get_string(ItemKey::ContentGroup).unwrap_or_default().into();
            f.subtitle = t.get_string(ItemKey::TrackSubtitle).unwrap_or_default().into();
            f.release_time = t
                .get_string(ItemKey::ReleaseDate)
                .or_else(|| t.get_string(ItemKey::OriginalReleaseDate))
                .unwrap_or_default()
                .into();
            f.podcast_category = t.get_string(ItemKey::PodcastSeriesCategory).unwrap_or_default().into();
            f.podcast_description = t.get_string(ItemKey::PodcastDescription).unwrap_or_default().into();
            f.podcast_id = t.get_string(ItemKey::PodcastGlobalUniqueId).unwrap_or_default().into();
            f.podcast_url = t.get_string(ItemKey::PodcastUrl).unwrap_or_default().into();
            f.show_title = t.get_string(ItemKey::ShowName).unwrap_or_default().into();
            f.episode_id = t.get_string(ItemKey::Description).unwrap_or_default().into();
            f.season_number = t.get_string(ItemKey::Work).unwrap_or_default().into();
            f.episode_number = t.get_string(ItemKey::Movement).unwrap_or_default().into();
            f.sort_title = t.get_string(ItemKey::TrackTitleSortOrder).unwrap_or_default().into();
            f.sort_artist = t.get_string(ItemKey::TrackArtistSortOrder).unwrap_or_default().into();
            f.sort_album_artist = t.get_string(ItemKey::AlbumArtistSortOrder).unwrap_or_default().into();
            f.sort_album = t.get_string(ItemKey::AlbumTitleSortOrder).unwrap_or_default().into();
            f.sort_composer = t.get_string(ItemKey::ComposerSortOrder).unwrap_or_default().into();
            f.sort_show = t.get_string(ItemKey::ShowNameSortOrder).unwrap_or_default().into();
            // The release ID and album ID are the same MusicBrainz
            // identifier (Picard's "Album Id"), so both boxes show it.
            f.mb_album_type = t.get_string(ItemKey::MusicBrainzReleaseType).unwrap_or_default().into();
            f.mb_album_artist_id = t.get_string(ItemKey::MusicBrainzReleaseArtistId).unwrap_or_default().into();
            f.mb_album_id = t.get_string(ItemKey::MusicBrainzReleaseId).unwrap_or_default().into();
            f.mb_artist_id = t.get_string(ItemKey::MusicBrainzArtistId).unwrap_or_default().into();
            f.mb_recording_id = t.get_string(ItemKey::MusicBrainzRecordingId).unwrap_or_default().into();
            f.mb_track_id = f.mb_recording_id.clone();
            f.mb_release_group_id = t.get_string(ItemKey::MusicBrainzReleaseGroupId).unwrap_or_default().into();
            f.mb_release_track_id = t.get_string(ItemKey::MusicBrainzTrackId).unwrap_or_default().into();
            f.mb_work_id = t.get_string(ItemKey::MusicBrainzWorkId).unwrap_or_default().into();
            f.rg_track_gain = t.get_string(ItemKey::ReplayGainTrackGain).unwrap_or_default().into();
            f.rg_track_peak = t.get_string(ItemKey::ReplayGainTrackPeak).unwrap_or_default().into();
            f.rg_album_gain = t.get_string(ItemKey::ReplayGainAlbumGain).unwrap_or_default().into();
            f.rg_album_peak = t.get_string(ItemKey::ReplayGainAlbumPeak).unwrap_or_default().into();
            f.compilation = t.get_string(ItemKey::FlagCompilation).map(|s| s == "1");
            // Keep a timestamp-looking year trimmed to four digits. Some
            // taggers write a v2.4 TDRC frame into a v2.3 tag; lofty skips
            // that, so read the raw frame before giving up on the year.
            f.year = trim_year(&f.year);
            if f.year.len() != 4 || !f.year.chars().all(|c| c.is_ascii_digit()) {
                if let Some(raw_year) = id3v2_year(path) {
                    f.year = raw_year;
                }
            }
            let cover = t.pictures().first().map(|p| {
                let mime = match p.mime_type() {
                    Some(MimeType::Jpeg) => "JPEG",
                    Some(MimeType::Png) => "PNG",
                    Some(MimeType::Bmp) => "BMP",
                    Some(MimeType::Gif) => "GIF",
                    _ => "Picture",
                };
                (mime.to_string(), std::sync::Arc::new(p.data().to_vec()))
            });
            (kind, f, cover)
        }
        None => (String::new(), TagFields::default(), None),
    };
    if tagged.file_type() == FileType::Mpeg {
        if let Ok(mut file) = std::fs::File::open(path) {
            if let Ok(mpeg) = MpegFile::read_from(&mut file, ParseOptions::new()) {
                if let Some(id3) = mpeg.id3v2() {
                    let text = |d: &str| id3.get_user_text(d).unwrap_or_default().to_string();
                    fields.mb_album_status = text(MB_ALBUM_STATUS_TXXX);
                    fields.mb_release_type = text(MB_RELEASE_TYPE_TXXX);
                    fields.mb_album_release_country = text(MB_ALBUM_RELEASE_COUNTRY_TXXX);
                }
            }
        }
    }

    Some(MusicFile {
        path: path.to_path_buf(),
        name: path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        is_dir: false,
        tag_kind,
        fields,
        duration_secs,
        bitrate_kbps,
        sample_rate,
        channels,
        bit_depth,
        size_bytes,
        modified_secs,
        // lofty's generic properties do not expose the mp3 VBR flag; the
        // original reads its own flag at item+0x34 (sub_ca0821).
        is_vbr: false,
        cover,
    })
}

/// Write the checked fields into the real file's tag and save it.
/// Returns an error message on failure (mirrors the original's
/// "Error while working with music file: ..." dialog).
/// lofty stores TrackNumber as a number, so "03" would come back as "3" in
/// an ID3v2 tag. This writes the TRCK frame as text so the zero is kept.
fn write_id3v2_track_text(path: &Path, track: &str) -> Result<(), String> {
    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut mpeg = MpegFile::read_from(&mut file, ParseOptions::new()).map_err(|e| e.to_string())?;
    drop(file);
    let Some(id3) = mpeg.id3v2_mut() else {
        return Ok(());
    };
    let id = FrameId::Valid(Cow::Borrowed("TRCK"));
    let _: Vec<_> = id3.remove(&id).collect();
    id3.insert(Frame::Text(TextInformationFrame::new(id, TextEncoding::UTF8, track.to_string())));
    mpeg.save_to_path(path, WriteOptions::new()).map_err(|e| e.to_string())
}

/// MusicBrainz values lofty has no key for; MP3 files keep them as ID3v2
/// user-text frames under these descriptions (the names Picard uses).
const MB_ALBUM_STATUS_TXXX: &str = "MusicBrainz Album Status";
const MB_RELEASE_TYPE_TXXX: &str = "MusicBrainz Release Type";
const MB_ALBUM_RELEASE_COUNTRY_TXXX: &str = "MusicBrainz Album Release Country";

/// Write the ticked user-text values into the ID3v2 tag; an empty value
/// removes its frame.
fn write_id3v2_user_text(path: &Path, items: &[(&str, &String, bool)]) -> Result<(), String> {
    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut mpeg = MpegFile::read_from(&mut file, ParseOptions::new()).map_err(|e| e.to_string())?;
    drop(file);
    let Some(id3) = mpeg.id3v2_mut() else {
        return Ok(());
    };
    for (desc, value, ticked) in items {
        if !*ticked {
            continue;
        }
        if value.is_empty() {
            id3.remove_user_text(desc);
        } else {
            id3.insert_user_text(desc.to_string(), value.to_string());
        }
    }
    mpeg.save_to_path(path, WriteOptions::new()).map_err(|e| e.to_string())
}

pub fn save_fields(path: &Path, fields: &TagFields, checked: &[Field]) -> Result<(), String> {
    let mut tagged: TaggedFile = Probe::open(path)
        .map_err(|e| e.to_string())?
        .read()
        .map_err(|e| e.to_string())?;

    // Make sure there is a primary tag to write into; the original creates
    // a tag of the file's native type (ID3v2 for mp3, Vorbis for ogg ...).
    if tagged.primary_tag().is_none() {
        let file_type = tagged.file_type();
        let tag = Tag::new(file_type.primary_tag_type());
        let _ = tagged.insert_tag(tag);
    }
    let tag = match tagged.primary_tag_mut() {
        Some(t) => t,
        None => return Err("no tag to write into".into()),
    };

    let has = |f: Field| checked.contains(&f);
    if has(Field::Title) {
        tag.set_title(fields.title.clone());
    }
    if has(Field::Artist) {
        tag.set_artist(fields.artist.clone());
    }
    if has(Field::Album) {
        tag.set_album(fields.album.clone());
    }
    if has(Field::AlbumArtist) {
        let _ = tag.insert_text(ItemKey::AlbumArtist, fields.album_artist.clone());
    }
    if has(Field::Year) {
        if let Ok(year) = fields.year.parse::<u16>() {
            tag.set_date(Timestamp {
                year,
                month: None,
                day: None,
                hour: None,
                minute: None,
                second: None,
            });
        }
    }
    if has(Field::Genre) {
        tag.set_genre(fields.genre.clone());
    }
    if has(Field::Comment) {
        tag.set_comment(fields.comment.clone());
    }
    // An empty value removes the tag item; a number is kept as typed, so
    // "01" stays "01" in formats that store text (an empty total is no
    // longer written as "/0").
    if has(Field::Track) {
        match leading_digits(&fields.track) {
            Some(v) => {
                let _ = tag.insert_text(ItemKey::TrackNumber, v);
            }
            None if fields.track.trim().is_empty() => tag.remove_key(ItemKey::TrackNumber),
            None => {
                let _ = tag.insert_text(ItemKey::TrackNumber, "0".to_string());
            }
        }
    }
    if has(Field::TrackTotal) {
        match leading_digits(&fields.track_total) {
            Some(v) => {
                let _ = tag.insert_text(ItemKey::TrackTotal, v);
            }
            None => tag.remove_key(ItemKey::TrackTotal),
        }
    }
    if has(Field::Disc) {
        let v = leading_number(&fields.disc).unwrap_or(0);
        tag.set_disk(v);
    }
    if has(Field::DiscTotal) {
        let v = leading_number(&fields.disc_total).unwrap_or(0);
        tag.set_disk_total(v);
    }
    for (field, key, value) in [
        (Field::Composer, ItemKey::Composer, &fields.composer),
        (Field::OrigArtist, ItemKey::OriginalArtist, &fields.orig_artist),
        (Field::Copyright, ItemKey::CopyrightMessage, &fields.copyright),
        (Field::Url, ItemKey::TrackArtistUrl, &fields.url),
        (Field::EncodedBy, ItemKey::EncodedBy, &fields.encoded_by),
        (Field::Mood, ItemKey::Mood, &fields.mood),
        (Field::Rating, ItemKey::Popularimeter, &fields.rating),
        (Field::Lyrics, ItemKey::UnsyncLyrics, &fields.lyrics),
        (Field::Conductor, ItemKey::Conductor, &fields.conductor),
        (Field::Lyricist, ItemKey::Lyricist, &fields.lyricist),
        (Field::Label, ItemKey::Label, &fields.label),
        (Field::RemixedBy, ItemKey::Remixer, &fields.remixed_by),
        (Field::Isrc, ItemKey::Isrc, &fields.isrc),
        (Field::Catalog, ItemKey::CatalogNumber, &fields.catalog),
        (Field::InitialKey, ItemKey::InitialKey, &fields.initial_key),
        (Field::OrigLyricist, ItemKey::OriginalLyricist, &fields.orig_lyricist),
        (Field::RadioName, ItemKey::InternetRadioStationName, &fields.radio_name),
        (Field::FilePage, ItemKey::AudioFileUrl, &fields.file_page),
        (Field::ArtistPage, ItemKey::TrackArtistUrl, &fields.artist_page),
        (Field::SourcePage, ItemKey::AudioSourceUrl, &fields.source_page),
        (Field::RadioPage, ItemKey::RadioStationUrl, &fields.radio_page),
        (Field::BuyPage, ItemKey::PaymentUrl, &fields.buy_page),
        (Field::Grouping, ItemKey::ContentGroup, &fields.grouping),
        (Field::Subtitle, ItemKey::TrackSubtitle, &fields.subtitle),
        (Field::ReleaseTime, ItemKey::ReleaseDate, &fields.release_time),
        (Field::PodcastCategory, ItemKey::PodcastSeriesCategory, &fields.podcast_category),
        (Field::PodcastDescription, ItemKey::PodcastDescription, &fields.podcast_description),
        (Field::PodcastId, ItemKey::PodcastGlobalUniqueId, &fields.podcast_id),
        (Field::PodcastUrl, ItemKey::PodcastUrl, &fields.podcast_url),
        (Field::ShowTitle, ItemKey::ShowName, &fields.show_title),
        (Field::EpisodeId, ItemKey::Description, &fields.episode_id),
        (Field::SeasonNumber, ItemKey::Work, &fields.season_number),
        (Field::EpisodeNumber, ItemKey::Movement, &fields.episode_number),
        (Field::SortTitle, ItemKey::TrackTitleSortOrder, &fields.sort_title),
        (Field::SortArtist, ItemKey::TrackArtistSortOrder, &fields.sort_artist),
        (Field::SortAlbumArtist, ItemKey::AlbumArtistSortOrder, &fields.sort_album_artist),
        (Field::SortAlbum, ItemKey::AlbumTitleSortOrder, &fields.sort_album),
        (Field::SortComposer, ItemKey::ComposerSortOrder, &fields.sort_composer),
        (Field::SortShow, ItemKey::ShowNameSortOrder, &fields.sort_show),
        (Field::MbAlbumType, ItemKey::MusicBrainzReleaseType, &fields.mb_album_type),
        (Field::MbAlbumArtistId, ItemKey::MusicBrainzReleaseArtistId, &fields.mb_album_artist_id),
        (Field::MbAlbumId, ItemKey::MusicBrainzReleaseId, &fields.mb_album_id),
        (Field::MbArtistId, ItemKey::MusicBrainzArtistId, &fields.mb_artist_id),
        (Field::MbRecordingId, ItemKey::MusicBrainzRecordingId, &fields.mb_recording_id),
        (Field::MbReleaseGroupId, ItemKey::MusicBrainzReleaseGroupId, &fields.mb_release_group_id),
        (Field::MbReleaseTrackId, ItemKey::MusicBrainzTrackId, &fields.mb_release_track_id),
        (Field::MbWorkId, ItemKey::MusicBrainzWorkId, &fields.mb_work_id),
        (Field::RgTrackGain, ItemKey::ReplayGainTrackGain, &fields.rg_track_gain),
        (Field::RgTrackPeak, ItemKey::ReplayGainTrackPeak, &fields.rg_track_peak),
        (Field::RgAlbumGain, ItemKey::ReplayGainAlbumGain, &fields.rg_album_gain),
        (Field::RgAlbumPeak, ItemKey::ReplayGainAlbumPeak, &fields.rg_album_peak),
    ] {
        if has(field) {
            if value.is_empty() {
                tag.remove_key(key);
            } else {
                let _ = tag.insert_text(key, value.clone());
            }
        }
    }
    // The track id and recording id are one MusicBrainz tag (Picard's
    // "MusicBrainz Track Id"); the recording field wins when both are ticked.
    if has(Field::MbTrackId) && !has(Field::MbRecordingId) {
        if fields.mb_track_id.is_empty() {
            tag.remove_key(ItemKey::MusicBrainzRecordingId);
        } else {
            let _ = tag.insert_text(ItemKey::MusicBrainzRecordingId, fields.mb_track_id.clone());
        }
    }
    if has(Field::Bpm) {
        if fields.bpm.is_empty() {
            tag.remove_key(ItemKey::IntegerBpm);
        } else {
            let v = leading_number(&fields.bpm).unwrap_or(0);
            let _ = tag.insert_text(ItemKey::IntegerBpm, v.to_string());
        }
    }
    if has(Field::Compilation) {
        match fields.compilation {
            Some(true) => {
                let _ = tag.insert_text(ItemKey::FlagCompilation, "1".to_string());
            }
            Some(false) => {
                tag.remove_key(ItemKey::FlagCompilation);
            }
            None => {}
        }
    }

    // Keep the file's timestamp unchanged when saving tags (the original's
    // "Don't change file timestamp on saving tags" option, on by default).
    let times = std::fs::metadata(path).ok().and_then(|m| {
        m.accessed()
            .ok()
            .and_then(|a| m.modified().ok().map(|mo| (a, mo)))
    });
    let res = tagged
        .save_to_path(path, WriteOptions::new())
        .map_err(|e| {
            format!(
                "Error while working with music file:\r\n{e}\r\nFile name: {}",
                path.display()
            )
        });
    let res = res.and_then(|_| match (has(Field::Track), leading_digits(&fields.track)) {
        (true, Some(v)) if v.len() > 1 && v.starts_with('0') && tagged.file_type() == FileType::Mpeg => {
            write_id3v2_track_text(path, &v)
        }
        _ => Ok(()),
    });
    let user_text = [
        (MB_ALBUM_STATUS_TXXX, &fields.mb_album_status, has(Field::MbAlbumStatus)),
        (MB_RELEASE_TYPE_TXXX, &fields.mb_release_type, has(Field::MbReleaseType)),
        (MB_ALBUM_RELEASE_COUNTRY_TXXX, &fields.mb_album_release_country, has(Field::MbAlbumReleaseCountry)),
    ];
    let res = res.and_then(|_| {
        if tagged.file_type() == FileType::Mpeg && user_text.iter().any(|t| t.2) {
            write_id3v2_user_text(path, &user_text)
        } else {
            Ok(())
        }
    });
    if let Some((a, m)) = times {
        restore_times(path, a, m);
    }
    res
}

fn leading_number(s: &str) -> Option<u32> {
    let digits: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

fn leading_digits(s: &str) -> Option<String> {
    let digits: String = s.trim().chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        None
    } else {
        Some(digits)
    }
}

#[cfg(unix)]
fn restore_times(path: &Path, atime: std::time::SystemTime, mtime: std::time::SystemTime) {
    use std::fs::FileTimes;
    let times = FileTimes::new().set_accessed(atime).set_modified(mtime);
    if let Ok(f) = std::fs::OpenOptions::new().write(true).open(path) {
        let _ = f.set_times(times);
    }
}

#[cfg(not(unix))]
fn restore_times(_path: &Path, _a: std::time::SystemTime, _m: std::time::SystemTime) {}

/// Remove every tag from the file (Remove Tags).
pub fn remove_tags(path: &Path) -> Result<(), String> {
    let mut tagged: TaggedFile = Probe::open(path)
        .map_err(|e| e.to_string())?
        .read()
        .map_err(|e| e.to_string())?;
    tagged.clear();
    tagged
        .save_to_path(path, WriteOptions::new())
        .map_err(|e| format!("Error while working with music file:\r\n{e}"))
}

fn clear_pictures(tag: &mut Tag) {
    while !tag.pictures().is_empty() {
        tag.remove_picture(0);
    }
}

fn mime_of(bytes: &[u8]) -> MimeType {
    match image::guess_format(bytes) {
        Ok(image::ImageFormat::Jpeg) => MimeType::Jpeg,
        Ok(image::ImageFormat::Png) => MimeType::Png,
        Ok(image::ImageFormat::Gif) => MimeType::Gif,
        Ok(image::ImageFormat::Bmp) => MimeType::Bmp,
        Ok(image::ImageFormat::WebP) => MimeType::Unknown("image/webp".into()),
        _ => MimeType::Png,
    }
}

/// Replace the embedded cover art of a file with the given picture bytes.
pub fn set_cover(path: &Path, data: Vec<u8>) -> Result<(), String> {
    let mime = mime_of(&data);
    let mut tagged: TaggedFile = Probe::open(path)
        .map_err(|e| e.to_string())?
        .read()
        .map_err(|e| e.to_string())?;
    let file_type = tagged.file_type();
    if tagged.primary_tag().is_none() {
        let tag = Tag::new(file_type.primary_tag_type());
        let _ = tagged.insert_tag(tag);
    }
    let tag = match tagged.primary_tag_mut() {
        Some(t) => t,
        None => return Err("no tag to write into".into()),
    };
    clear_pictures(tag);
    let pic = Picture::unchecked(data)
        .pic_type(PictureType::CoverFront)
        .mime_type(mime)
        .build();
    tag.push_picture(pic);
    tagged
        .save_to_path(path, WriteOptions::new())
        .map_err(|e| e.to_string())
}

/// Remove the embedded cover art from a file.
pub fn remove_cover(path: &Path) -> Result<(), String> {
    let mut tagged: TaggedFile = Probe::open(path)
        .map_err(|e| e.to_string())?
        .read()
        .map_err(|e| e.to_string())?;
    if let Some(tag) = tagged.primary_tag_mut() {
        clear_pictures(tag);
    }
    tagged
        .save_to_path(path, WriteOptions::new())
        .map_err(|e| e.to_string())
}

/// A plain sample row for the unit tests below (not shown in the app).
#[cfg(test)]
pub fn sample_file() -> MusicFile {
    let fields = TagFields {
        title: "Northern Lights".into(),
        artist: "Aurora Skyline".into(),
        album: "Polar Drift".into(),
        year: "2024".into(),
        track: "1".into(),
        track_total: "10".into(),
        genre: "Ambient".into(),
        ..TagFields::default()
    };
    MusicFile {
        path: PathBuf::from("Sample/01 - Northern Lights.mp3"),
        name: "01 - Northern Lights.mp3".into(),
        is_dir: false,
        tag_kind: "ID3v2".into(),
        fields,
        duration_secs: 254.0,
        bitrate_kbps: 320,
        sample_rate: 44100,
        channels: 2,
        bit_depth: None,
        size_bytes: 254 * 320 * 128,
        modified_secs: 0,
        is_vbr: false,
        cover: None,
    }
}

#[cfg(test)]
mod track_zero_tests {
    use super::*;

    fn id3v24_with_track(track: &[u8]) -> Vec<u8> {
        let mut body = vec![3u8];
        body.extend_from_slice(track);
        let mut frame = b"TRCK".to_vec();
        frame.extend_from_slice(&[0, 0, 0, body.len() as u8]);
        frame.extend_from_slice(&[0, 0]);
        frame.extend_from_slice(&body);
        let mut out = b"ID3\x04\x00\x00".to_vec();
        out.extend_from_slice(&[0, 0, 0, (frame.len() as u8) & 0x7f]);
        out.extend_from_slice(&frame);
        out
    }

    #[test]
    fn mp3_track_keeps_its_leading_zero() {
        let path = std::env::temp_dir().join("tagrename-track-zero-test.mp3");
        std::fs::write(&path, id3v24_with_track(b"01/12")).unwrap();
        assert_eq!(id3v2_track_number(&path).as_deref(), Some("01"));
        std::fs::write(&path, id3v24_with_track(b"3")).unwrap();
        assert_eq!(id3v2_track_number(&path).as_deref(), Some("3"));
        let _ = std::fs::remove_file(&path);
    }

    fn id3v23_with_text_frame(id: &[u8; 4], text: &[u8]) -> Vec<u8> {
        let mut body = vec![0u8];
        body.extend_from_slice(text);
        body.push(0);
        let mut frame = id.to_vec();
        frame.extend_from_slice(&(body.len() as u32).to_be_bytes());
        frame.extend_from_slice(&[0, 0]);
        frame.extend_from_slice(&body);
        let mut out = b"ID3\x03\x00\x00".to_vec();
        out.extend_from_slice(&[
            0,
            ((frame.len() >> 14) & 0x7f) as u8,
            ((frame.len() >> 7) & 0x7f) as u8,
            (frame.len() & 0x7f) as u8,
        ]);
        out.extend_from_slice(&frame);
        out
    }

    #[test]
    fn mp3_year_reads_full_timestamp_from_tdrc() {
        let path = std::env::temp_dir().join("tagrename-year-tdrc-test.mp3");
        std::fs::write(&path, id3v23_with_text_frame(b"TDRC", b"2003-01-01")).unwrap();
        assert_eq!(id3v2_year(&path).as_deref(), Some("2003"));
        std::fs::write(&path, id3v23_with_text_frame(b"TYER", b"1987")).unwrap();
        assert_eq!(id3v2_year(&path).as_deref(), Some("1987"));
        let _ = std::fs::remove_file(&path);
    }
}

#[cfg(test)]
mod track_save_tests {
    use super::*;

    #[test]
    fn saved_track_keeps_its_leading_zero_in_mp3() {
        let path = std::env::temp_dir().join("tagrename-track-save-test.mp3");
        let mut bytes = b"ID3\x04\x00\x00\x00\x00\x00\x00".to_vec();
        let mut frame = b"TRCK".to_vec();
        frame.extend_from_slice(&[0, 0, 0, 2, 0, 0, 3, b'1']);
        bytes[9] = frame.len() as u8;
        bytes.extend_from_slice(&frame);
        for _ in 0..400 {
            bytes.extend_from_slice(&[0xFF, 0xFB, 0x90, 0x00]);
            bytes.extend_from_slice(&[0u8; 413]);
        }
        std::fs::write(&path, &bytes).unwrap();
        let mut fields = read_file(&path).unwrap().fields;
        assert_eq!(fields.track, "1");
        fields.track = "03".into();
        save_fields(&path, &fields, &[Field::Track]).unwrap();
        assert_eq!(read_file(&path).unwrap().fields.track, "03");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn musicbrainz_fields_round_trip_in_mp3() {
        let path = std::env::temp_dir().join("tagrename-musicbrainz-test.mp3");
        let mut bytes = b"ID3\x04\x00\x00\x00\x00\x00\x00".to_vec();
        let mut frame = b"TRCK".to_vec();
        frame.extend_from_slice(&[0, 0, 0, 2, 0, 0, 3, b'1']);
        bytes[9] = frame.len() as u8;
        bytes.extend_from_slice(&frame);
        for _ in 0..400 {
            bytes.extend_from_slice(&[0xFF, 0xFB, 0x90, 0x00]);
            bytes.extend_from_slice(&[0u8; 413]);
        }
        std::fs::write(&path, &bytes).unwrap();
        let fields = TagFields {
            mb_album_type: "album".into(),
            mb_album_artist_id: "aa-1".into(),
            mb_album_id: "rel-1".into(),
            mb_album_status: "official".into(),
            mb_artist_id: "art-1".into(),
            mb_recording_id: "rec-1".into(),
            mb_release_group_id: "grp-1".into(),
            mb_release_track_id: "trk-1".into(),
            mb_release_type: "album".into(),
            mb_album_release_country: "GB".into(),
            mb_work_id: "work-1".into(),
            ..TagFields::default()
        };
        let checked = [
            Field::MbAlbumType,
            Field::MbAlbumArtistId,
            Field::MbAlbumId,
            Field::MbAlbumStatus,
            Field::MbArtistId,
            Field::MbRecordingId,
            Field::MbReleaseGroupId,
            Field::MbReleaseTrackId,
            Field::MbReleaseType,
            Field::MbAlbumReleaseCountry,
            Field::MbWorkId,
        ];
        save_fields(&path, &fields, &checked).unwrap();
        let back = read_file(&path).unwrap().fields;
        assert_eq!(back.mb_album_type, "album");
        assert_eq!(back.mb_album_artist_id, "aa-1");
        assert_eq!(back.mb_album_id, "rel-1");
        assert_eq!(back.mb_album_status, "official");
        assert_eq!(back.mb_artist_id, "art-1");
        assert_eq!(back.mb_recording_id, "rec-1");
        assert_eq!(back.mb_track_id, "rec-1");
        assert_eq!(back.mb_release_group_id, "grp-1");
        assert_eq!(back.mb_release_track_id, "trk-1");
        assert_eq!(back.mb_release_type, "album");
        assert_eq!(back.mb_album_release_country, "GB");
        assert_eq!(back.mb_work_id, "work-1");
        let _ = std::fs::remove_file(&path);
    }
}
