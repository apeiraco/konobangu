use std::collections::HashMap;

use lazy_static::lazy_static;
use regex::Regex;

use super::types::*;

#[derive(Debug, Clone)]
pub struct KeywordEntry {
    pub word: &'static str,
    pub labels: &'static [Label],
    pub canonical: Option<&'static str>,
}

#[derive(Debug)]
pub struct CompiledRegex {
    pub regex: Regex,
    pub labels: &'static [Label],
}

pub struct KeywordDb {
    keywords: HashMap<String, Vec<&'static KeywordEntry>>,
    regex_entries: Vec<CompiledRegex>,
}

impl KeywordDb {
    pub fn find<'a>(&'a self, token: &str) -> impl Iterator<Item = &'a &'static KeywordEntry> + 'a {
        let normalized = token.to_uppercase();
        self.keywords
            .get(&normalized)
            .into_iter()
            .flat_map(|v| v.iter())
    }

    pub fn find_regex_matches(
        &self,
        input: &str,
        token_start: usize,
        token_end: usize,
    ) -> Vec<(Label, usize, usize)> {
        let mut results = Vec::new();
        let upper = input.to_uppercase();
        for entry in &self.regex_entries {
            for m in entry.regex.find_iter(&upper) {
                if m.start() <= token_start && token_end <= m.end() {
                    for &label in entry.labels {
                        results.push((label, m.start(), m.end()));
                    }
                }
            }
        }
        results
    }
}

macro_rules! kw {
    ($word:expr, $($label:expr),+) => {
        KeywordEntry { word: $word, labels: &[$($label),+], canonical: None }
    };
    ($word:expr, $($label:expr),+ ; canonical = $c:expr) => {
        KeywordEntry { word: $word, labels: &[$($label),+], canonical: Some($c) }
    };
}

static AUDIO_TERMS: &[KeywordEntry] = &[
    kw!("AAC", Label::AudioTerm),
    kw!("AC3", Label::AudioTerm),
    kw!("DTS", Label::AudioTerm),
    kw!("EAC3", Label::AudioTerm),
    kw!("FLAC", Label::AudioTerm),
    kw!("LOSSLESS", Label::AudioTerm),
    kw!("MP3", Label::AudioTerm),
    kw!("OGG", Label::AudioTerm),
    kw!("VORBIS", Label::AudioTerm),
    kw!("OPUS", Label::AudioTerm),
    kw!("TRUEHD", Label::AudioTerm),
    kw!("PCM", Label::AudioTerm),
    kw!("ALAC", Label::AudioTerm),
    kw!("WMA", Label::AudioTerm),
    kw!("ATMOS", Label::AudioTerm),
    kw!("DUALAUDIO", Label::AudioTerm),
    kw!("MULTIAUDIO", Label::AudioTerm),
];

static VIDEO_TERMS: &[KeywordEntry] = &[
    kw!("HEVC", Label::VideoTerm),
    kw!("DIVX", Label::VideoTerm),
    kw!("WMV", Label::VideoTerm),
    kw!("HQ", Label::VideoTerm),
    kw!("LQ", Label::VideoTerm),
    kw!("HD", Label::VideoTerm),
    kw!("SD", Label::VideoTerm),
    kw!("AVI", Label::VideoTerm),
    kw!("AV1", Label::VideoTerm),
    kw!("RMVB", Label::VideoTerm),
    kw!("AVC", Label::VideoTerm),
    kw!("XVID", Label::VideoTerm),
    kw!("X264", Label::VideoTerm),
    kw!("X265", Label::VideoTerm),
    kw!("H264", Label::VideoTerm),
    kw!("H265", Label::VideoTerm),
    kw!("VP9", Label::VideoTerm),
    kw!("VP8", Label::VideoTerm),
    kw!("MPEG4", Label::VideoTerm),
    kw!("MPEG2", Label::VideoTerm),
    kw!("THEORA", Label::VideoTerm),
    kw!("PRORES", Label::VideoTerm),
    kw!("HDR", Label::VideoTerm),
    kw!("HDR10", Label::VideoTerm),
    kw!("DV", Label::VideoTerm),
    kw!("8BIT", Label::VideoTerm),
    kw!("10BIT", Label::VideoTerm),
    kw!("12BIT", Label::VideoTerm),
    kw!("HI10P", Label::VideoTerm),
    kw!("HI10", Label::VideoTerm),
    kw!("HI444PP", Label::VideoTerm),
    kw!("MP4", Label::VideoTerm),
    kw!("MKV", Label::VideoTerm),
];

static SOURCES: &[KeywordEntry] = &[
    kw!("UHDRIP", Label::Source),
    kw!("BD", Label::Source),
    kw!("BDRIP", Label::Source),
    kw!("BLURAY", Label::Source),
    kw!("DVD", Label::Source),
    kw!("DVDRIP", Label::Source),
    kw!("DVDSCR", Label::Source),
    kw!("REMUX", Label::Source),
    kw!("RIP", Label::Source),
    kw!("HDTV", Label::Source),
    kw!("TVRIP", Label::Source),
    kw!("WEBRIP", Label::Source),
    kw!("WEBDL", Label::Source),
    kw!("AMZN", Label::Source),
    kw!("NF", Label::Source),
    kw!("DSNP", Label::Source),
    kw!("HIDIVE", Label::Source),
    kw!("HIDI", Label::Source),
    kw!("FUNI", Label::Source),
    kw!("VRV", Label::Source),
    kw!("HULU", Label::Source),
    kw!("MAX", Label::Source),
    kw!("ABEMA", Label::Source),
    kw!("ADN", Label::Source),
    kw!("BILI", Label::Source),
    kw!("BAHA", Label::Source),
    kw!("NETFLIX", Label::Source),
    kw!("TTFC", Label::Source),
    kw!("B-GLOBAL", Label::Source),
    kw!("WEBCAST", Label::Source),
];

static EPISODE_PREFIXES: &[KeywordEntry] = &[
    kw!("第", Label::EpisodePrefix),
    kw!("EP", Label::EpisodePrefix),
    kw!("EPS", Label::EpisodePrefix),
    kw!("EPISODE", Label::EpisodePrefix),
    kw!("EPISODES", Label::EpisodePrefix),
    kw!("CAPITULO", Label::EpisodePrefix),
    kw!("EPISODIO", Label::EpisodePrefix),
    kw!("FOLGE", Label::EpisodePrefix),
    kw!("E", Label::EpisodePrefix),
    kw!("#", Label::EpisodePrefix),
];

static SEASON_PREFIXES: &[KeywordEntry] = &[
    kw!("SAISON", Label::SeasonPrefix),
    kw!("SEASON", Label::SeasonPrefix),
    kw!("S", Label::SeasonPrefix),
    kw!("COUR", Label::SeasonPrefix),
    kw!("TEMPORADA", Label::SeasonPrefix),
    kw!("STAFFEL", Label::SeasonPrefix),
];

static VOLUME_PREFIXES: &[KeywordEntry] = &[
    kw!("VOL", Label::VolumePrefix),
    kw!("VOLUME", Label::VolumePrefix),
];

static SERIES_TYPES: &[KeywordEntry] = &[
    kw!("剧场版", Label::SeriesType),
    kw!("GEKIJOUBAN", Label::SeriesType),
    kw!("MOVIE", Label::SeriesType),
    kw!("MOVIES", Label::SeriesType; canonical = "Movie"),
    kw!("OAD", Label::SeriesType),
    kw!("OAV", Label::SeriesType),
    kw!("OVAS", Label::SeriesType; canonical = "OVA"),
    kw!("ONA", Label::SeriesType),
    kw!("OVA", Label::SeriesType),
    kw!("SPECIAL", Label::SeriesType),
    kw!("SPECIALS", Label::SeriesType; canonical = "Special"),
    kw!("TV", Label::SeriesType),
    kw!("SP", Label::SeriesType),
];

static CONTENT_TYPES: &[KeywordEntry] = &[
    kw!("ED", Label::ContentType),
    kw!("ENDING", Label::ContentType),
    kw!("NCED", Label::ContentType),
    kw!("OP", Label::ContentType),
    kw!("OPENING", Label::ContentType),
    kw!("NCOP", Label::ContentType),
    kw!("PREVIEW", Label::ContentType),
    kw!("PV", Label::ContentType),
    kw!("PILOT", Label::ContentType),
];

static RELEASE_INFO: &[KeywordEntry] = &[
    kw!("BATCH", Label::ReleaseInformation),
    kw!("COMPLETE", Label::ReleaseInformation),
    kw!("PATCH", Label::ReleaseInformation),
    kw!("END", Label::ReleaseInformation),
    kw!("FIN", Label::ReleaseInformation),
    kw!("FINAL", Label::ReleaseInformation),
    kw!("REMASTER", Label::ReleaseInformation),
    kw!("REMASTERED", Label::ReleaseInformation),
    kw!("UNCENSORED", Label::ReleaseInformation),
    kw!("UNCUT", Label::ReleaseInformation),
    kw!("TS", Label::ReleaseInformation),
    kw!("VFR", Label::ReleaseInformation),
    kw!("WIDESCREEN", Label::ReleaseInformation),
    kw!("WS", Label::ReleaseInformation),
];

static RELEASE_VERSIONS: &[KeywordEntry] = &[
    kw!("VER", Label::ReleaseVersion),
    kw!("VERSION", Label::ReleaseVersion),
];

static SUBS_TERMS: &[KeywordEntry] = &[
    kw!("ASS", Label::SubsTerm),
    kw!("BIG5", Label::SubsTerm),
    kw!("DUB", Label::SubsTerm),
    kw!("DUBBED", Label::SubsTerm),
    kw!("HARDSUB", Label::SubsTerm),
    kw!("HARDSUBS", Label::SubsTerm),
    kw!("RAW", Label::SubsTerm),
    kw!("SOFTSUB", Label::SubsTerm),
    kw!("SOFTSUBS", Label::SubsTerm),
    kw!("SUB", Label::SubsTerm),
    kw!("SUBBED", Label::SubsTerm),
    kw!("SUBTITLE", Label::SubsTerm),
    kw!("SUBTITLES", Label::SubsTerm),
    kw!("SUBTITLED", Label::SubsTerm),
    kw!("SUBS", Label::SubsTerm),
    kw!("SRT", Label::SubsTerm),
    kw!("VTT", Label::SubsTerm),
    kw!("PGS", Label::SubsTerm),
    kw!("CC", Label::SubsTerm),
    kw!("DUALSUB", Label::SubsTerm),
    kw!("DUALSUBS", Label::SubsTerm),
    kw!("MULTISUB", Label::SubsTerm),
    kw!("MULTISUBS", Label::SubsTerm),
];

static LANGUAGES: &[KeywordEntry] = &[
    // ISO 639-2
    kw!("ENG", Label::Language),
    kw!("JPN", Label::Language),
    kw!("JAP", Label::Language),
    kw!("CHI", Label::Language),
    kw!("KOR", Label::Language),
    kw!("SPA", Label::Language),
    kw!("FRE", Label::Language),
    kw!("FRA", Label::Language),
    kw!("GER", Label::Language),
    kw!("DEU", Label::Language),
    kw!("ITA", Label::Language),
    kw!("POR", Label::Language),
    kw!("RUS", Label::Language),
    kw!("ARA", Label::Language),
    kw!("THA", Label::Language),
    kw!("VIE", Label::Language),
    kw!("POL", Label::Language),
    kw!("DUT", Label::Language),
    kw!("HUN", Label::Language),
    kw!("TUR", Label::Language),
    kw!("GRE", Label::Language),
    kw!("HEB", Label::Language),
    kw!("HIN", Label::Language),
    kw!("SWE", Label::Language),
    kw!("NOR", Label::Language),
    kw!("UKR", Label::Language),
    kw!("CZE", Label::Language),
    kw!("RON", Label::Language),
    kw!("FIN", Label::Language),
    kw!("DAN", Label::Language),
    kw!("PER", Label::Language),
    // Full names
    kw!("ENGLISH", Label::Language),
    kw!("JAPANESE", Label::Language),
    kw!("CHINESE", Label::Language),
    kw!("KOREAN", Label::Language),
    kw!("SPANISH", Label::Language),
    kw!("FRENCH", Label::Language),
    kw!("GERMAN", Label::Language),
    kw!("ITALIAN", Label::Language),
    kw!("PORTUGUESE", Label::Language),
    kw!("RUSSIAN", Label::Language),
    // Short codes
    kw!("EN", Label::Language),
    kw!("JP", Label::Language),
    kw!("CH", Label::Language),
    kw!("FR", Label::Language),
    kw!("DE", Label::Language),
    kw!("RU", Label::Language),
    kw!("PL", Label::Language),
    kw!("NL", Label::Language),
    kw!("PT", Label::Language),
    kw!("ZH", Label::Language),
    kw!("SC", Label::Language),
    kw!("TC", Label::Language),
    kw!("CHS", Label::Language),
    kw!("CHT", Label::Language),
    kw!("GB", Label::Language),
    kw!("MANDARIN", Label::Language),
    kw!("CANTONESE", Label::Language),
    kw!("IND", Label::Language),
    kw!("MAL", Label::Language),
    kw!("VOSTFR", Label::Language),
    kw!("PTBR", Label::Language),
    kw!("ESP", Label::Language),
    kw!("DEUTSCH", Label::Language),
    // CJK language descriptors
    kw!("简日双语", Label::Language),
    kw!("繁日双语", Label::Language),
    kw!("简繁日内封字幕", Label::Language),
    kw!("简繁内封字幕", Label::Language),
    kw!("简日内封字幕", Label::Language),
    kw!("简繁字幕", Label::Language),
    kw!("简日双语字幕", Label::Language),
    kw!("简体字幕", Label::Language),
    kw!("繁体字幕", Label::Language),
    kw!("中文字幕", Label::Language),
    kw!("中文配", Label::Language),
    kw!("中文配音", Label::Language),
    kw!("台粤配", Label::Language),
    kw!("国语", Label::Language),
    kw!("國語", Label::Language),
    kw!("粤语", Label::Language),
    kw!("粵語", Label::Language),
    kw!("中配版", Label::Language),
    kw!("中配", Label::Language),
    kw!("国语版", Label::Language),
    kw!("國語版", Label::Language),
    kw!("日语中字", Label::Language),
    kw!("繁体", Label::Language),
];

static DEVICE_COMPAT: &[KeywordEntry] = &[
    kw!("IPAD", Label::DeviceCompatibility),
    kw!("IPHONE", Label::DeviceCompatibility),
    kw!("IPOD", Label::DeviceCompatibility),
    kw!("ANDROID", Label::DeviceCompatibility),
];

static CONTEXT_DEPENDENTS: &[KeywordEntry] = &[
    kw!("THE", Label::ContextDependent),
    kw!("PART", Label::ContextDependent),
    kw!("+", Label::ContextDependent),
    kw!("&", Label::ContextDependent),
];

static MISC_KEYWORDS: &[KeywordEntry] = &[
    kw!("月新番", Label::ReleaseInformation),
    kw!("新番", Label::ReleaseInformation),
    kw!("★", Label::ReleaseInformation),
    kw!("招募翻译", Label::Other),
    kw!("招募", Label::Other),
    kw!("生", Label::SubsTerm),
    kw!("CR", Label::ReleaseInformation),
];

static REGEX_PATTERNS: &[(&str, &[Label])] = &[
    // Video resolution
    (r"\d{3,4}[PpIi]", &[Label::VideoResolution]),
    (r"\d[Kk]", &[Label::VideoResolution]),
    (r"\d{3,4}[Xx×]\d{3,4}", &[Label::VideoResolution]),
    // Video term codecs
    (r"[Hh][\W_]?26[45]", &[Label::VideoTerm]),
    (r"[Xx]\.?26[45]", &[Label::VideoTerm]),
    (r"HEVC2?", &[Label::VideoTerm]),
    (r"AV[\W_]?1", &[Label::VideoTerm]),
    (r"DOLBY[\W_]?VISION", &[Label::VideoTerm]),
    (r"\d{1,3}(?:\.\d{1,3})?FPS", &[Label::VideoTerm]),
    (r"\d{1,2}[\W_]?BITS?", &[Label::VideoTerm]),
    // Audio terms
    (r"[2-9](?:\.\d{1,2}){1,2}(?:CH)?", &[Label::AudioTerm]),
    (r"DTS(?:[\W_]?(?:ES|HD[\W_]?(?:MA|HR)?))?", &[Label::AudioTerm]),
    (r"TRUE[\W_]?HD", &[Label::AudioTerm]),
    (r"AC[\W_]?3", &[Label::AudioTerm]),
    (r"E[\W_]?AC[\W_]?3", &[Label::AudioTerm]),
    (r"DD[\W_]?\d{1,2}(?:\.\d{1,2}){0,2}", &[Label::AudioTerm]),
    (r"FLAC\d\.\d", &[Label::AudioTerm]),
    (r"HE[\W_]?AAC", &[Label::AudioTerm]),
    (r"DUAL[\W_]?AUDIO", &[Label::AudioTerm]),
    (r"MULTI[\W_]?AUDIO", &[Label::AudioTerm]),
    // Source
    (r"UHD[\W_]?RIP", &[Label::Source]),
    (r"BD[\W_]?RIP", &[Label::Source]),
    (r"BLU[\W_]?RAY", &[Label::Source]),
    (r"DVD[\W_]?(?:R\d{1,2}|R(?:E?MUX|IP))", &[Label::Source]),
    (r"TV[\W_]?RIP", &[Label::Source]),
    (r"WEB[\W_]?(?:RIP|CAST|DL)", &[Label::Source]),
    // Episode prefix with optional dot
    (r"EP\.?\d+", &[Label::EpisodePrefix]),
    // SxE pattern
    (r"\d+[Xx]\d+", &[Label::EpisodePrefix]),
    // Release version
    (r"[Vv]\d{1,2}", &[Label::ReleaseVersion]),
    // File checksum
    (r"(?<![A-Za-z0-9])[A-Fa-f0-9]{8}(?![A-Za-z0-9])", &[Label::FileChecksum]),
    // Subtitle compound
    (r"MULTIPLE[\W_]?(?:SUBTITLES?|SUBS?)", &[Label::SubsTerm]),
    (r"DUAL[\W_]?(?:SUBTITLES?|SUBS?)", &[Label::SubsTerm]),
    (r"MULTI[\W_]?(?:SUBTITLES?|SUBS?)", &[Label::SubsTerm]),
    (r"SOFT[\W_]?SUBS?", &[Label::SubsTerm]),
    // Device compat
    (r"IPAD\d+", &[Label::DeviceCompatibility]),
    (r"IPHONE\d+", &[Label::DeviceCompatibility]),
    (r"PS\d+", &[Label::DeviceCompatibility]),
    (r"XBOX\d+", &[Label::DeviceCompatibility]),
    // Volume prefix with optional dot
    (r"VOL\.?\d+", &[Label::VolumePrefix]),
    // Content type compound
    (r"CLEAN[\W_]?(?:ENDING|OPENING)S?", &[Label::ContentType]),
    // Complete version
    (r"COMPLETE[\W_]?VERSION", &[Label::ReleaseInformation]),
    // HEVC-10bit compound
    (r"HEVC[\W_]?10[\W_]?BIT", &[Label::VideoTerm]),
];

fn build_db() -> KeywordDb {
    let all_entries: &[&[KeywordEntry]] = &[
        AUDIO_TERMS,
        VIDEO_TERMS,
        SOURCES,
        EPISODE_PREFIXES,
        SEASON_PREFIXES,
        VOLUME_PREFIXES,
        SERIES_TYPES,
        CONTENT_TYPES,
        RELEASE_INFO,
        RELEASE_VERSIONS,
        SUBS_TERMS,
        LANGUAGES,
        DEVICE_COMPAT,
        CONTEXT_DEPENDENTS,
        MISC_KEYWORDS,
    ];

    let mut keywords: HashMap<String, Vec<&'static KeywordEntry>> = HashMap::new();
    for entries in all_entries {
        for entry in *entries {
            keywords
                .entry(entry.word.to_uppercase())
                .or_default()
                .push(entry);
        }
    }

    let regex_entries: Vec<CompiledRegex> = REGEX_PATTERNS
        .iter()
        .filter_map(|(pattern, labels)| {
            let full_pattern = format!("(?i){}", pattern);
            Regex::new(&full_pattern).ok().map(|regex| CompiledRegex {
                regex,
                labels,
            })
        })
        .collect();

    KeywordDb {
        keywords,
        regex_entries,
    }
}

lazy_static! {
    pub static ref DEFAULT_DB: KeywordDb = build_db();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_keyword() {
        let db = &*DEFAULT_DB;
        let results: Vec<_> = db.find("hevc").collect();
        assert!(!results.is_empty());
        assert!(results[0].labels.contains(&Label::VideoTerm));
    }

    #[test]
    fn test_find_case_insensitive() {
        let db = &*DEFAULT_DB;
        let results: Vec<_> = db.find("HEVC").collect();
        assert!(!results.is_empty());

        let results2: Vec<_> = db.find("Hevc").collect();
        assert!(!results2.is_empty());
    }

    #[test]
    fn test_find_source() {
        let db = &*DEFAULT_DB;
        let results: Vec<_> = db.find("WebRip").collect();
        assert!(!results.is_empty());
        assert!(results[0].labels.contains(&Label::Source));
    }

    #[test]
    fn test_regex_resolution() {
        let db = &*DEFAULT_DB;
        let matches = db.find_regex_matches("1080p", 0, 5);
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|(l, _, _)| *l == Label::VideoResolution));
    }

    #[test]
    fn test_regex_codec() {
        let db = &*DEFAULT_DB;
        let matches = db.find_regex_matches("x264", 0, 4);
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|(l, _, _)| *l == Label::VideoTerm));
    }

    #[test]
    fn test_cjk_keywords() {
        let db = &*DEFAULT_DB;
        let results: Vec<_> = db.find("简日双语").collect();
        assert!(!results.is_empty());
        assert!(results[0].labels.contains(&Label::Language));
    }
}
