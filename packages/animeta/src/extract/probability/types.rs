use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TokenClass {
    Cjk,
    Latin,
    Number,
    Symbol,
    Delimiter,
    BracketOpen,
    BracketClose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Label {
    Unknown,
    Title,
    SeasonPrefix,
    EpisodePrefix,
    VolumePrefix,
    SequenceNumber,
    SequenceRange,
    Season,
    Episode,
    Volume,
    ContentType,
    ContentIdentifier,
    SeriesType,
    Year,
    ReleaseGroup,
    Source,
    VideoResolution,
    VideoTerm,
    AudioTerm,
    SubsTerm,
    Language,
    ReleaseInformation,
    ReleaseVersion,
    DeviceCompatibility,
    FileChecksum,
    FileExtension,
    FileIndex,
    Bracket,
    Delimiter,
    ContextDelimiter,
    ContextDependent,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LabelGroup {
    Unknown,
    Title,
    SequencePrefix,
    SequenceNumber,
    SequenceRange,
    Season,
    Episode,
    Volume,
    ContentIdentifier,
    SeriesType,
    Year,
    ReleaseGroup,
    Source,
    VideoResolution,
    VideoTerm,
    AudioTerm,
    SubsTerm,
    Language,
    ReleaseInformation,
    ReleaseVersion,
    DeviceCompatibility,
    FileChecksum,
    FileExtension,
    FileIndex,
    Bracket,
    Delimiter,
    ContextDelimiter,
    ContextDependent,
    Other,
}

impl Label {
    pub fn group(self) -> LabelGroup {
        match self {
            Label::Unknown => LabelGroup::Unknown,
            Label::Title => LabelGroup::Title,
            Label::SeasonPrefix | Label::EpisodePrefix | Label::VolumePrefix => {
                LabelGroup::SequencePrefix
            }
            Label::SequenceNumber => LabelGroup::SequenceNumber,
            Label::SequenceRange => LabelGroup::SequenceRange,
            Label::Season => LabelGroup::Season,
            Label::Episode => LabelGroup::Episode,
            Label::Volume => LabelGroup::Volume,
            Label::ContentType => LabelGroup::SequencePrefix,
            Label::ContentIdentifier => LabelGroup::ContentIdentifier,
            Label::SeriesType => LabelGroup::SeriesType,
            Label::Year => LabelGroup::Year,
            Label::ReleaseGroup => LabelGroup::ReleaseGroup,
            Label::Source => LabelGroup::Source,
            Label::VideoResolution => LabelGroup::VideoResolution,
            Label::VideoTerm => LabelGroup::VideoTerm,
            Label::AudioTerm => LabelGroup::AudioTerm,
            Label::SubsTerm => LabelGroup::SubsTerm,
            Label::Language => LabelGroup::Language,
            Label::ReleaseInformation => LabelGroup::ReleaseInformation,
            Label::ReleaseVersion => LabelGroup::ReleaseVersion,
            Label::DeviceCompatibility => LabelGroup::DeviceCompatibility,
            Label::FileChecksum => LabelGroup::FileChecksum,
            Label::FileExtension => LabelGroup::FileExtension,
            Label::FileIndex => LabelGroup::FileIndex,
            Label::Bracket => LabelGroup::Bracket,
            Label::Delimiter => LabelGroup::Delimiter,
            Label::ContextDelimiter => LabelGroup::ContextDelimiter,
            Label::ContextDependent => LabelGroup::ContextDependent,
            Label::Other => LabelGroup::Other,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoreReason {
    pub rule: &'static str,
    pub delta: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Possibility {
    pub label: Label,
    pub score: i32,
    pub reasons: Vec<ScoreReason>,
    pub element_ref: Option<usize>,
}

impl Possibility {
    pub fn new(label: Label, base_score: i32) -> Self {
        Self {
            label,
            score: base_score,
            reasons: Vec::new(),
            element_ref: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Zone {
    Title,
    Transition,
    Metadata,
}

#[derive(Debug, Clone, Serialize)]
pub struct Token {
    pub span: Span,
    pub content: String,
    pub class: TokenClass,
    pub bracket_depth: u32,
    pub zone: Zone,
    pub bracket_group: Option<usize>,
    pub possibilities: HashMap<Label, Possibility>,
    pub resolved: Option<Label>,
    pub split_boundary: bool,
}

impl Token {
    pub fn new(span: Span, content: String, class: TokenClass) -> Self {
        Self {
            span,
            content,
            class,
            bracket_depth: 0,
            zone: Zone::Title,
            bracket_group: None,
            possibilities: HashMap::new(),
            resolved: None,
            split_boundary: false,
        }
    }

    pub fn add_possibility(&mut self, label: Label, base_score: i32) {
        self.possibilities
            .entry(label)
            .and_modify(|p| p.score = base_score)
            .or_insert_with(|| Possibility::new(label, base_score));
    }

    pub fn remove_possibility(&mut self, label: Label) {
        self.possibilities.remove(&label);
    }

    pub fn add_score(&mut self, label: Label, delta: i32, rule_name: &'static str) {
        if let Some(p) = self.possibilities.get_mut(&label) {
            p.score += delta;
            p.reasons.push(ScoreReason {
                rule: rule_name,
                delta,
            });
        }
    }

    pub fn get_score(&self, label: Label) -> i32 {
        self.possibilities
            .get(&label)
            .map_or(0, |p| p.score)
    }

    pub fn get_best_possibility(&self) -> Option<(Label, &Possibility)> {
        self.possibilities
            .iter()
            .max_by_key(|(_, p)| p.score)
            .map(|(&label, p)| (label, p))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChunkKind {
    Plain,
    Bracketed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChunkPosition {
    Leading,
    Middle,
    Trailing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub span: Span,
    pub kind: ChunkKind,
    pub position: ChunkPosition,
    pub token_start: usize,
    pub token_end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BracketGroup {
    pub group_index: usize,
    pub open_pos: usize,
    pub close_pos: usize,
    pub is_first: bool,
    pub is_last: bool,
    pub starts_filename: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelimiterProfile {
    pub primary: Option<char>,
    pub counts: HashMap<char, usize>,
    pub is_uniform: bool,
    pub file_index_end: Option<usize>,
    pub bracket_groups: Vec<BracketGroup>,
}
