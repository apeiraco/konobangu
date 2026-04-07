use std::borrow::Cow;

use itertools::Itertools;
use lazy_static::lazy_static;
use ::nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{is_a, tag, tag_no_case, take_till},
    character::complete::{anychar, char as chartag, none_of, space0, space1},
    combinator::{map, opt, recognize, value, verify},
    multi::{many_m_n, many_till, many0, many1},
    number::complete::float,
    sequence::{delimited, preceded, terminated},
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::{Level, instrument};

use crate::nom::{
    ZhNum, delimited_by_brackets, is_han_scx, parse_int, parse_month_num, parse_uint,
    with_recognized,
};

const BARKET_ALL: &str = "[【(（]））】";
lazy_static! {
    static ref NAME_CLEAR_RE: Regex =
        Regex::new(r"[\[\]【】][ ]?[\[\]【】]?|[ ][\[\]【】]?").unwrap();
}

pub trait OriginCompTrait<'a>: Sized {
    fn parse_comp(input: &'a str) -> IResult<&'a str, Self>;
    fn into_source_string(self) -> String;
    fn as_source_str(&self) -> &str;
}

pub type EpisodeNum = i32;

pub struct EpisodeComp<'a> {
    pub source: Cow<'a, str>,
    pub num: EpisodeNum,
    pub num2: Option<EpisodeNum>,
}

impl<'a> EpisodeComp<'a> {
    fn parse_ep_round_num(input: &'a str) -> IResult<&'a str, i32> {
        let (input, num) = float(input)?;
        Ok((input, f32::round(num) as i32))
    }

    fn parse_ep_special_num(input: &'a str) -> IResult<&'a str, i32> {
        terminated(
            alt((
                value(0, tag_no_case("ova")),
                value(0, tag_no_case("oad")),
                value(0, tag_no_case("sp")),
                value(0, tag_no_case("ex")),
            )),
            (space0, opt(parse_int::<i32>)),
        )
        .parse(input)
    }

    fn parse_ep_num(input: &'a str) -> IResult<&'a str, i32> {
        alt((
            parse_int::<i32>,
            Self::parse_ep_round_num,
            ZhNum::parse_int,
            Self::parse_ep_special_num,
        ))
        .parse(input)
    }

    fn parse_ep_nums_core(input: &'a str) -> IResult<&'a str, (i32, Option<i32>)> {
        delimited(
            space0,
            (
                delimited(space0, Self::parse_ep_num, space0),
                opt(preceded(
                    is_a("-~"),
                    delimited(space0, Self::parse_ep_num, space0),
                )),
            ),
            (opt((tag_no_case("v"), parse_uint::<u32>)), space0),
        )
        .parse(input)
    }

    fn parse_with_ep_prefix(input: &'a str) -> IResult<&'a str, (i32, Option<i32>)> {
        preceded(tag_no_case("ep"), Self::parse_ep_nums_core).parse(input)
    }

    fn parse_with_zh_suffix(input: &'a str) -> IResult<&'a str, (i32, Option<i32>)> {
        delimited(
            opt(tag("第")),
            Self::parse_ep_nums_core,
            alt((tag("话"), tag("集"), tag("話"))),
        )
        .parse(input)
    }

    fn parse_with_collection_suffix(input: &'a str) -> IResult<&'a str, (i32, Option<i32>)> {
        let collection_zh = |input| -> IResult<&str, &str> {
            recognize(many_till(is_han_scx, tag("合集"))).parse(input)
        };

        let collection_en = tag_no_case("end");

        terminated(
            Self::parse_ep_nums_core,
            alt((collection_zh, collection_en)),
        )
        .parse(input)
    }

    fn parse_with_delimited_buckets(input: &'a str) -> IResult<&'a str, (i32, Option<i32>)> {
        delimited(
            is_a("[【"),
            delimited(
                space0,
                alt((
                    Self::parse_with_ep_prefix,
                    Self::parse_with_zh_suffix,
                    Self::parse_with_collection_suffix,
                    Self::parse_ep_nums_core,
                )),
                space0,
            ),
            is_a("】]"),
        )
        .parse(input)
    }

    fn parse_with_prefix_hyphen(input: &'a str) -> IResult<&'a str, (i32, Option<i32>)> {
        preceded(
            delimited(space0, is_a("-"), space1),
            alt((
                Self::parse_with_ep_prefix,
                Self::parse_with_zh_suffix,
                Self::parse_with_collection_suffix,
                Self::parse_ep_nums_core,
            )),
        )
        .parse(input)
    }
}

impl<'a> std::fmt::Debug for EpisodeComp<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_source_str())
    }
}

impl<'a> OriginCompTrait<'a> for EpisodeComp<'a> {
    #[cfg_attr(debug_assertions, instrument(level = Level::TRACE, ret, err(level=Level::TRACE), "EpisodeComp::parse_comp"))]
    fn parse_comp(input: &'a str) -> IResult<&'a str, Self> {
        let (input, ((num, num2), source)) = with_recognized(alt((
            Self::parse_with_delimited_buckets,
            Self::parse_with_prefix_hyphen,
            Self::parse_with_ep_prefix,
            Self::parse_with_zh_suffix,
            Self::parse_with_collection_suffix,
        )))
        .parse(input)?;

        Ok((
            input,
            Self {
                source: source.into(),
                num,
                num2,
            },
        ))
    }

    fn into_source_string(self) -> String {
        self.source.into()
    }

    fn as_source_str(&self) -> &str {
        self.source.as_ref()
    }
}

pub struct MoiveComp<'a> {
    pub source: Cow<'a, str>,
}

impl<'a> std::fmt::Debug for MoiveComp<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_source_str())
    }
}

impl<'a> OriginCompTrait<'a> for MoiveComp<'a> {
    #[cfg_attr(debug_assertions, instrument(level = Level::TRACE, ret, err(level=Level::TRACE), "MoiveComp::parse_comp"))]
    fn parse_comp(input: &'a str) -> IResult<&'a str, Self> {
        let (input, source) = alt((
            tag("剧场版"),
            tag("电影"),
            tag_no_case("movie"),
            tag_no_case("film"),
        ))
        .parse(input)?;
        Ok((
            input,
            Self {
                source: source.into(),
            },
        ))
    }

    fn into_source_string(self) -> String {
        self.source.into()
    }

    fn as_source_str(&self) -> &str {
        self.source.as_ref()
    }
}

pub struct FansubComp<'a> {
    pub source: Cow<'a, str>,
}

impl<'a> std::fmt::Debug for FansubComp<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_source_str())
    }
}

impl<'a> OriginCompTrait<'a> for FansubComp<'a> {
    #[cfg_attr(debug_assertions, instrument(level = Level::TRACE, ret, err(level=Level::TRACE), "FansubComp::parse_comp"))]
    fn parse_comp(input: &'a str) -> IResult<&'a str, Self> {
        let (input, source) = delimited(space0, delimited_by_brackets, space0).parse(input)?;

        Ok((
            input,
            Self {
                source: source.into(),
            },
        ))
    }

    fn into_source_string(self) -> String {
        self.source.into()
    }

    fn as_source_str(&self) -> &str {
        self.source.as_ref()
    }
}

pub type SeasonNum = i32;

pub struct SeasonComp<'a> {
    pub source: Cow<'a, str>,
    pub num: SeasonNum,
}

impl<'a> SeasonComp<'a> {
    fn parse_season_round_num(input: &'a str) -> IResult<&'a str, SeasonNum> {
        let (input, num) = float(input)?;

        Ok((input, f32::round(num) as i32))
    }

    fn parse_season_num(input: &'a str) -> IResult<&'a str, SeasonNum> {
        alt((
            parse_uint::<i32>,
            Self::parse_season_round_num,
            ZhNum::parse_int,
        ))
        .parse(input)
    }

    fn parse_season_prefix(input: &'a str) -> IResult<&'a str, SeasonNum> {
        preceded(
            alt((tag("S"), tag_no_case("season"))),
            Self::parse_season_num,
        )
        .parse(input)
    }

    fn parse_en123_ordinial(input: &'a str) -> IResult<&'a str, SeasonNum> {
        alt((
            value(1, tag_no_case("1st")),
            value(2, tag_no_case("2nd")),
            value(3, tag_no_case("3rd")),
        ))
        .parse(input)
    }

    fn parse_en4plus_ordinial(input: &'a str) -> IResult<&'a str, SeasonNum> {
        terminated(Self::parse_season_num, tag_no_case("th")).parse(input)
    }

    fn parse_zh_pattern(input: &'a str) -> IResult<&'a str, SeasonNum> {
        delimited(
            opt(tag("第")),
            Self::parse_season_num,
            alt((tag("季"), tag("期"), tag("部分"))),
        )
        .parse(input)
    }
}

impl<'a> std::fmt::Debug for SeasonComp<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_source_str())
    }
}

impl<'a> OriginCompTrait<'a> for SeasonComp<'a> {
    #[cfg_attr(debug_assertions, instrument(level = Level::TRACE, ret, err(level=Level::TRACE), "SeasonComp::parse_comp"))]
    fn parse_comp(input: &'a str) -> IResult<&'a str, Self> {
        let (input, (num, source)) = with_recognized(alt((
            Self::parse_season_prefix,
            Self::parse_en123_ordinial,
            Self::parse_en4plus_ordinial,
            Self::parse_zh_pattern,
        )))
        .parse(input)?;

        Ok((
            input,
            Self {
                source: source.into(),
                num,
            },
        ))
    }

    fn into_source_string(self) -> String {
        self.source.into()
    }

    fn as_source_str(&self) -> &str {
        self.source.as_ref()
    }
}

pub struct ResolutionComp<'a> {
    pub source: Cow<'a, str>,
    pub keyword: Cow<'a, str>,
}

impl<'a> std::fmt::Debug for ResolutionComp<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_source_str())
    }
}

impl<'a> OriginCompTrait<'a> for ResolutionComp<'a> {
    #[cfg_attr(debug_assertions, instrument(level = Level::TRACE, ret, err(level=Level::TRACE), "ResolutionComp::parse_comp"))]
    fn parse_comp(input: &'a str) -> IResult<&'a str, Self> {
        let (input, ((_, keyword), source)) = with_recognized(terminated(
            many_till(
                anychar,
                alt((
                    tag("720"),
                    tag("1080"),
                    tag("1440"),
                    tag("2160"),
                    tag("3840"),
                    tag_no_case("2K"),
                    tag_no_case("4K"),
                    tag_no_case("8K"),
                )),
            ),
            many0(anychar),
        ))
        .parse(input)?;
        Ok((
            input,
            Self {
                source: source.into(),
                keyword: keyword.into(),
            },
        ))
    }

    fn into_source_string(self) -> String {
        self.source.into()
    }

    fn as_source_str(&self) -> &str {
        self.source.as_ref()
    }
}

pub struct SubtitleComp<'a> {
    pub source: Cow<'a, str>,
    pub keyword: Cow<'a, str>,
}

impl<'a> std::fmt::Debug for SubtitleComp<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_source_str())
    }
}

impl<'a> OriginCompTrait<'a> for SubtitleComp<'a> {
    #[cfg_attr(debug_assertions, instrument(level = Level::TRACE, ret, err(level=Level::TRACE), "SubtitleComp::parse_comp"))]
    fn parse_comp(input: &'a str) -> IResult<&'a str, Self> {
        let (input, ((_, keyword), source)) = verify(
            with_recognized(terminated(
                many_till(
                    anychar,
                    alt((
                        tag_no_case("ch"),
                        tag_no_case("big5"),
                        tag_no_case("gb"),
                        tag("简"),
                        tag("繁"),
                        tag("日"),
                        tag("字幕"),
                        tag("内封"),
                        tag("翻译"),
                        tag("中字"),
                        tag("英字"),
                        tag("生"),
                    )),
                ),
                many0(anychar),
            )),
            |(_, s)| !s.contains("招人") && !s.contains("招募"),
        )
        .parse(input)?;

        Ok((
            input,
            Self {
                source: source.into(),
                keyword: keyword.into(),
            },
        ))
    }

    fn into_source_string(self) -> String {
        self.source.into()
    }

    fn as_source_str(&self) -> &str {
        self.source.as_ref()
    }
}

pub struct SourceL1Comp<'a> {
    pub source: Cow<'a, str>,
    pub keyword: Cow<'a, str>,
}

impl<'a> std::fmt::Debug for SourceL1Comp<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_source_str())
    }
}

impl<'a> OriginCompTrait<'a> for SourceL1Comp<'a> {
    #[cfg_attr(debug_assertions, instrument(level = Level::TRACE, ret, err(level=Level::TRACE), "SourceL1Comp::parse_comp"))]
    fn parse_comp(input: &'a str) -> IResult<&'a str, Self> {
        let (input, ((_, keyword), source)) = with_recognized(terminated(
            many_till(
                anychar,
                alt((
                    tag_no_case("b-global"),
                    tag_no_case("baha"),
                    tag_no_case("bilibili"),
                    tag_no_case("at-x"),
                    tag_no_case("webrip"),
                    tag_no_case("sentai"),
                    tag_no_case("bdrip"),
                    tag_no_case("uhdrip"),
                    tag_no_case("netflix"),
                )),
            ),
            many0(anychar),
        ))
        .parse(input)?;
        Ok((
            input,
            Self {
                source: source.into(),
                keyword: keyword.into(),
            },
        ))
    }

    fn into_source_string(self) -> String {
        self.source.into()
    }

    fn as_source_str(&self) -> &str {
        self.source.as_ref()
    }
}

pub struct SourceL2Comp<'a> {
    pub source: Cow<'a, str>,
    pub keyword: Cow<'a, str>,
}

impl<'a> std::fmt::Debug for SourceL2Comp<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_source_str())
    }
}

impl<'a> OriginCompTrait<'a> for SourceL2Comp<'a> {
    #[cfg_attr(debug_assertions, instrument(level = Level::TRACE, ret, err(level=Level::TRACE), "SourceL2Comp::parse_comp"))]
    fn parse_comp(input: &'a str) -> IResult<&'a str, Self> {
        let (input, ((_, keyword), source)) = with_recognized(terminated(
            many_till(
                anychar,
                alt((tag("AMZ"), tag("CR"), tag_no_case("web"), tag_no_case("bd"))),
            ),
            many0(anychar),
        ))
        .parse(input)?;
        Ok((
            input,
            Self {
                source: source.into(),
                keyword: keyword.into(),
            },
        ))
    }

    fn into_source_string(self) -> String {
        self.source.into()
    }

    fn as_source_str(&self) -> &str {
        self.source.as_ref()
    }
}

pub struct RegionLimitComp<'a> {
    pub source: Cow<'a, str>,
    pub keyword: Cow<'a, str>,
}

impl<'a> std::fmt::Debug for RegionLimitComp<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_source_str())
    }
}

impl<'a> OriginCompTrait<'a> for RegionLimitComp<'a> {
    #[cfg_attr(debug_assertions, instrument(level = Level::TRACE, ret, err(level=Level::TRACE), "RegionLimitComp::parse_comp"))]
    fn parse_comp(input: &'a str) -> IResult<&'a str, Self> {
        let (input, ((_, keyword), source)) = with_recognized(terminated(
            many_till(
                verify(none_of(BARKET_ALL), |c| !c.is_whitespace()),
                recognize((
                    tag("仅限"),
                    many_m_n(1, 3, is_a("港澳台")),
                    opt(tag("地区")),
                )),
            ),
            take_till(|c: char| c.is_whitespace() || BARKET_ALL.contains(c)),
        ))
        .parse(input)?;
        Ok((
            input,
            Self {
                source: source.into(),
                keyword: keyword.into(),
            },
        ))
    }

    fn into_source_string(self) -> String {
        self.source.into()
    }

    fn as_source_str(&self) -> &str {
        self.source.as_ref()
    }
}

pub struct SeasonDescComp<'a> {
    pub source: Cow<'a, str>,
    pub keyword: Cow<'a, str>,
}

impl<'a> SeasonDescComp<'a> {
    fn parse_core_month(input: &str) -> IResult<&str, &str> {
        recognize((parse_month_num, chartag('月'))).parse(input)
    }

    fn parse_core_keyword(input: &str) -> IResult<&str, &str> {
        recognize((is_a("春夏秋冬"), opt(chartag('季')))).parse(input)
    }

    fn parse_core(input: &str) -> IResult<&str, &str> {
        alt((Self::parse_core_month, Self::parse_core_keyword)).parse(input)
    }

    fn parse_with_suffix(input: &str) -> IResult<&str, &str> {
        recognize((Self::parse_core, space0, opt(chartag('新')), chartag('番'))).parse(input)
    }

    fn parse_with_backets(input: &str) -> IResult<&str, &str> {
        delimited(
            is_a("[【(（"),
            terminated(
                map(
                    many_till(
                        none_of(BARKET_ALL),
                        alt((Self::parse_with_suffix, Self::parse_core)),
                    ),
                    |(_, v)| v,
                ),
                take_till(|c: char| BARKET_ALL.contains(c)),
            ),
            is_a("]））】"),
        )
        .parse(input)
    }

    #[instrument(level = Level::TRACE, ret, err(level=Level::TRACE), "SeasonDescComp::parse_without_backets")]
    fn parse_without_backets(input: &str) -> IResult<&str, &str> {
        terminated(
            map(
                many_till(
                    verify(none_of(BARKET_ALL), |c| !c.is_whitespace()),
                    alt((Self::parse_with_suffix, Self::parse_core)),
                ),
                |(_, v)| v,
            ),
            take_till(|c: char| c.is_whitespace() || BARKET_ALL.contains(c)),
        )
        .parse(input)
    }
}

impl<'a> std::fmt::Debug for SeasonDescComp<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_source_str())
    }
}

impl<'a> OriginCompTrait<'a> for SeasonDescComp<'a> {
    #[cfg_attr(debug_assertions, instrument(level = Level::TRACE, ret, err(level=Level::TRACE), "SeasonDescComp::parse_comp"))]
    fn parse_comp(input: &'a str) -> IResult<&'a str, Self> {
        let (input, (keyword, source)) =
            with_recognized(alt((Self::parse_with_backets, Self::parse_without_backets)))
                .parse(input)?;
        Ok((
            input,
            Self {
                source: source.into(),
                keyword: keyword.into(),
            },
        ))
    }

    fn into_source_string(self) -> String {
        self.source.into()
    }

    fn as_source_str(&self) -> &str {
        self.source.as_ref()
    }
}
pub struct BangumiComps<'a> {
    pub source: Cow<'a, str>,
    pub season_desc: Option<SeasonDescComp<'a>>,
    pub region_limit: Option<RegionLimitComp<'a>>,
    pub season: Option<SeasonComp<'a>>,
    pub name: Cow<'a, str>,
}

impl<'a> std::fmt::Debug for BangumiComps<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_source_str())
    }
}

impl<'a> OriginCompTrait<'a> for BangumiComps<'a> {
    #[cfg_attr(debug_assertions, instrument(level = Level::TRACE, ret, err(level=Level::TRACE), "BangumiComps::parse_comp"))]
    fn parse_comp(input: &'a str) -> IResult<&'a str, Self> {
        let (main, (season_desc, region_limit)) = (
            opt(SeasonDescComp::parse_comp),
            opt(RegionLimitComp::parse_comp),
        )
            .parse(input)?;

        let season = many_till(anychar, SeasonComp::parse_comp)
            .parse(main)
            .ok()
            .map(|(_, (_, season))| season);

        let name = NAME_CLEAR_RE.replace_all(main, " ").trim().to_string();

        Ok((
            "",
            Self {
                source: input.into(),
                season,
                name: name.into(),
                season_desc,
                region_limit,
            },
        ))
    }

    fn into_source_string(self) -> String {
        self.source.into()
    }

    fn as_source_str(&self) -> &str {
        self.source.as_ref()
    }
}

pub struct ExtraComps<'a> {
    pub source: Cow<'a, str>,
    pub resolution: Option<ResolutionComp<'a>>,
    pub sub: Option<SubtitleComp<'a>>,
    pub source_l1: Option<SourceL1Comp<'a>>,
    pub source_l2: Option<SourceL2Comp<'a>>,
    pub region_limit: Option<RegionLimitComp<'a>>,
}

impl<'a> std::fmt::Debug for ExtraComps<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_source_str())
    }
}

impl<'a> OriginCompTrait<'a> for ExtraComps<'a> {
    #[cfg_attr(debug_assertions, instrument(level = Level::TRACE, ret, err(level=Level::TRACE), "ExtraComps::parse_comp"))]
    fn parse_comp(input: &'a str) -> IResult<&'a str, Self> {
        let splitted = input
            .split(['[', ']', '【', '】', '(', ')', '（', '）', '_', ' '])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect_vec();

        let mut sub: Option<SubtitleComp> = None;
        let mut resolution: Option<ResolutionComp> = None;
        let mut source_l1: Option<SourceL1Comp> = None;
        let mut source_l2: Option<SourceL2Comp> = None;
        let mut region_limit: Option<RegionLimitComp> = None;

        for elem in splitted.iter() {
            if let Ok((_, s)) = SubtitleComp::parse_comp(elem) {
                sub = Some(s);
            } else if let Ok((_, s)) = ResolutionComp::parse_comp(elem) {
                resolution = Some(s);
            } else if let Ok((_, s)) = SourceL1Comp::parse_comp(elem) {
                source_l1 = Some(s);
            } else if let Ok((_, s)) = RegionLimitComp::parse_comp(elem) {
                region_limit = Some(s);
            }
        }
        if source_l1.is_none() {
            for element in splitted.iter() {
                if let Ok((_, s)) = SourceL2Comp::parse_comp(element) {
                    source_l2 = Some(s);
                }
            }
        }

        Ok((
            input,
            Self {
                source: input.into(),
                resolution,
                sub,
                source_l1,
                source_l2,
                region_limit,
            },
        ))
    }

    fn into_source_string(self) -> String {
        self.source.into()
    }

    fn as_source_str(&self) -> &str {
        self.source.as_ref()
    }
}

pub struct OriginNameEpisode<'a> {
    pub source: Cow<'a, str>,
    pub fansub: Option<FansubComp<'a>>,
    pub bangumi: BangumiComps<'a>,
    pub episode: EpisodeComp<'a>,
    pub extras: ExtraComps<'a>,
}

impl<'a> std::fmt::Debug for OriginNameEpisode<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_source_str())
    }
}

impl<'a> OriginCompTrait<'a> for OriginNameEpisode<'a> {
    #[cfg_attr(debug_assertions, instrument(level = Level::TRACE, ret, err(level=Level::TRACE), "OriginEpisode::parse_comp"))]
    fn parse_comp(input: &'a str) -> IResult<&'a str, Self> {
        let (fansub_remain, fansub) = opt(FansubComp::parse_comp).parse(input)?;

        let (extra_input, (bangumi_input, episode)) =
            map(many_till(anychar, EpisodeComp::parse_comp), |(pre, ep)| {
                (
                    &fansub_remain[..pre.into_iter().collect::<String>().len()],
                    ep,
                )
            })
            .parse(fansub_remain)?;

        let (_, bangumi) = BangumiComps::parse_comp(bangumi_input)?;
        let (_, extras) = ExtraComps::parse_comp(extra_input)?;

        Ok((
            input,
            Self {
                source: input.into(),
                fansub,
                bangumi,
                episode,
                extras,
            },
        ))
    }

    fn into_source_string(self) -> String {
        self.source.into()
    }

    fn as_source_str(&self) -> &str {
        self.source.as_ref()
    }
}

impl<'a> From<OriginNameEpisode<'a>> for OriginNameMeta {
    fn from(val: OriginNameEpisode<'a>) -> Self {
        OriginNameMeta {
            name: val.bangumi.name.into(),
            season: val.bangumi.season.as_ref().map_or(1, |s| s.num),
            season_raw: val.bangumi.season.map(|s| s.into_source_string()),
            episode_index: val.episode.num,
            subtitle: val.extras.sub.map(|s| s.into_source_string()),
            source: val
                .extras
                .source_l1
                .map(|s| s.into_source_string())
                .or(val.extras.source_l2.map(|s| s.into_source_string())),
            fansub: val.fansub.map(|s| s.into_source_string()),
            resolution: val.extras.resolution.map(|s| s.into_source_string()),
        }
    }
}

pub struct OriginNameMovie<'a> {
    pub source: Cow<'a, str>,
    pub fansub: Option<FansubComp<'a>>,
    pub movie: MoiveComp<'a>,
    pub bangumi: BangumiComps<'a>,
    pub extras: ExtraComps<'a>,
}

impl<'a> From<OriginNameMovie<'a>> for OriginNameMeta {
    fn from(val: OriginNameMovie<'a>) -> Self {
        OriginNameMeta {
            name: val.bangumi.name.into(),
            season: val.bangumi.season.as_ref().map_or(1, |s| s.num),
            season_raw: val.bangumi.season.map(|s| s.into_source_string()),
            episode_index: 1,
            subtitle: val.extras.sub.map(|s| s.into_source_string()),
            source: val
                .extras
                .source_l1
                .map(|s| s.into_source_string())
                .or(val.extras.source_l2.map(|s| s.into_source_string())),
            fansub: val.fansub.map(|s| s.into_source_string()),
            resolution: val.extras.resolution.map(|s| s.into_source_string()),
        }
    }
}

impl<'a> std::fmt::Debug for OriginNameMovie<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_source_str())
    }
}

impl<'a> OriginCompTrait<'a> for OriginNameMovie<'a> {
    #[cfg_attr(debug_assertions, instrument(level = Level::TRACE, ret, err(level=Level::TRACE), "OriginMovie::parse_comp"))]
    fn parse_comp(input: &'a str) -> IResult<&'a str, Self> {
        let (fansub_remain, fansub) = opt(FansubComp::parse_comp).parse(input)?;

        let (extra_input, (mut movies, bangumi_input)) = with_recognized(many1(map(
            many_till(anychar, MoiveComp::parse_comp),
            |(_, movie)| movie,
        )))
        .parse(fansub_remain)?;

        let (_, extras) = ExtraComps::parse_comp(extra_input)?;

        let (_, bangumi) = BangumiComps::parse_comp(bangumi_input)?;

        Ok((
            input,
            Self {
                source: input.into(),
                fansub,
                bangumi,
                movie: movies.pop().unwrap(),
                extras,
            },
        ))
    }

    fn into_source_string(self) -> String {
        self.source.into()
    }

    fn as_source_str(&self) -> &str {
        self.source.as_ref()
    }
}

pub enum OriginNameRoot<'a> {
    Movie(OriginNameMovie<'a>),
    Episode(OriginNameEpisode<'a>),
}

impl<'a> OriginNameRoot<'a> {
    fn parse_movie(input: &'a str) -> IResult<&'a str, Self> {
        let (input, movie) = OriginNameMovie::parse_comp(input)?;
        Ok((input, Self::Movie(movie)))
    }

    fn parse_episode(input: &'a str) -> IResult<&'a str, Self> {
        let (input, episode) = OriginNameEpisode::parse_comp(input)?;
        Ok((input, Self::Episode(episode)))
    }

    pub fn into_meta(self) -> OriginNameMeta {
        OriginNameMeta::from(self)
    }
}

impl<'a> std::fmt::Debug for OriginNameRoot<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_source_str())
    }
}

impl<'a> OriginCompTrait<'a> for OriginNameRoot<'a> {
    #[cfg_attr(debug_assertions, instrument(level = Level::TRACE, ret, err(level=Level::TRACE), "OriginData::parse_comp"))]
    fn parse_comp(input: &'a str) -> IResult<&'a str, Self> {
        alt((Self::parse_movie, Self::parse_episode)).parse(input)
    }

    fn into_source_string(self) -> String {
        match self {
            Self::Movie(m) => m.into_source_string(),
            Self::Episode(e) => e.into_source_string(),
        }
    }

    fn as_source_str(&self) -> &str {
        match self {
            Self::Movie(m) => m.as_source_str(),
            Self::Episode(e) => e.as_source_str(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OriginNameMeta {
    pub name: String,
    pub season: i32,
    pub season_raw: Option<String>,
    pub episode_index: i32,
    pub subtitle: Option<String>,
    pub source: Option<String>,
    pub fansub: Option<String>,
    pub resolution: Option<String>,
}

impl<'a> From<OriginNameRoot<'a>> for OriginNameMeta {
    fn from(val: OriginNameRoot<'a>) -> Self {
        match val {
            OriginNameRoot::Movie(m) => m.into(),
            OriginNameRoot::Episode(e) => e.into(),
        }
    }
}

#[cfg(test)]
#[allow(unused_variables)]
mod tests {
    use rstest::{fixture, rstest};
    use anyhow::Result;

    use crate::extract::legacy::{
        OriginCompTrait, OriginNameMeta, OriginNameRoot,
    };

    fn test_parse_origin_data(origin_name: &str, expected: &str) -> Result<()> {
        let (_, data) = OriginNameRoot::parse_comp(origin_name).map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let found: OriginNameMeta = data.into();
        let expected: OriginNameMeta = serde_json::from_str(expected).inspect_err(|e| {
            tracing::error!(
                "Failed to parse expected: {}, but got found: {}",
                e,
                serde_json::to_string_pretty(&found).unwrap()
            );
        })?;

        if expected != found {
            println!(
                "expected {} and found {} are not equal",
                serde_json::to_string_pretty(&expected).unwrap(),
                serde_json::to_string_pretty(&found).unwrap()
            )
        }
        assert_eq!(expected, found);

        Ok(())
    }

    #[fixture]
    fn before_each() {
    }

    #[rstest]
    #[test]
    fn test_parse_ep_with_all_parts_wrapped(before_each: ()) -> Result<()> {
        test_parse_origin_data(
            r#"[新Sub][1月新番][我心里危险的东西 第二季][05][HEVC][10Bit][1080P][简日双语][招募翻译]"#,
            r#"{
                "name": "我心里危险的东西 第二季",
                "season": 2,
                "season_raw": "第二季",
                "episode_index": 5,
                "subtitle": "简日双语",
                "source": null,
                "fansub": "新Sub",
                "resolution": "1080P"
                }"#,
        )
    }

    #[rstest]
    #[test]
    fn test_parse_ep_with_title_wrapped_by_one_square_bracket_and_season_prefix(
        before_each: (),
    ) -> Result<()> {
        test_parse_origin_data(
            r#"【喵萌奶茶屋】★01月新番★[我内心的糟糕念头 / Boku no Kokoro no Yabai Yatsu][18][1080p][简日双语][招募翻译]"#,
            r#"{
                  "name": "我内心的糟糕念头 / Boku no Kokoro no Yabai Yatsu",
                  "season": 1,
                  "season_raw": null,
                  "episode_index": 18,
                  "subtitle": "简日双语",
                  "source": null,
                  "fansub": "喵萌奶茶屋",
                  "resolution": "1080p"
                }"#,
        )
    }

    #[rstest]
    #[test]
    fn test_parse_ep_with_ep_and_version(before_each: ()) -> Result<()> {
        test_parse_origin_data(
            r#"[LoliHouse] 因为不是真正的伙伴而被逐出勇者队伍，流落到边境展开慢活人生 2nd / Shin no Nakama 2nd - 08v2 [WebRip 1080p HEVC-10bit AAC][简繁内封字幕]"#,
            r#"{
                    "name": "因为不是真正的伙伴而被逐出勇者队伍，流落到边境展开慢活人生 2nd / Shin no Nakama 2nd",
                    "season": 2,
                    "season_raw": "2nd",
                    "episode_index": 8,
                    "subtitle": "简繁内封字幕",
                    "source": "WebRip",
                    "fansub": "LoliHouse",
                    "resolution": "1080p"
                  }"#,
        )
    }

    #[rstest]
    #[test]
    fn test_parse_ep_with_en_title_only(before_each: ()) -> Result<()> {
        test_parse_origin_data(
            r"[动漫国字幕组&LoliHouse] THE MARGINAL SERVICE - 08 [WebRip 1080p HEVC-10bit AAC][简繁内封字幕]",
            r#"{
                "name": "THE MARGINAL SERVICE",
                "season": 1,
                "episode_index": 8,
                "subtitle": "简繁内封字幕",
                "source": "WebRip",
                "fansub": "动漫国字幕组&LoliHouse",
                "resolution": "1080p"
              }"#,
        )
    }

    #[rstest]
    #[test]
    fn test_parse_ep_with_two_zh_title(before_each: ()) -> Result<()> {
        test_parse_origin_data(
            r#"[LoliHouse] 事与愿违的不死冒险者 / 非自愿的不死冒险者 / Nozomanu Fushi no Boukensha - 01 [WebRip 1080p HEVC-10bit AAC][简繁内封字幕]"#,
            r#"{
                    "name": "事与愿违的不死冒险者 / 非自愿的不死冒险者 / Nozomanu Fushi no Boukensha",
                    "season": 1,
                    "episode_index": 1,
                    "subtitle": "简繁内封字幕",
                    "source": "WebRip",
                    "fansub": "LoliHouse",
                    "resolution": "1080p"
                  }"#,
        )
    }

    #[rstest]
    #[test]
    fn test_parse_ep_with_en_zh_jp_titles(before_each: ()) -> Result<()> {
        test_parse_origin_data(
            r#"[喵萌奶茶屋&LoliHouse] 碰之道 / ぽんのみち / Pon no Michi - 07 [WebRip 1080p HEVC-10bit AAC][简繁日内封字幕]"#,
            r#"{
                    "name": "碰之道 / ぽんのみち / Pon no Michi",
                    "season": 1,
                    "episode_index": 7,
                    "subtitle": "简繁日内封字幕",
                    "source": "WebRip",
                    "fansub": "喵萌奶茶屋&LoliHouse",
                    "resolution": "1080p"
                }"#,
        )
    }

    #[rstest]
    #[test]
    fn test_parse_ep_with_nth_season(before_each: ()) -> Result<()> {
        test_parse_origin_data(
            r#"[ANi] Yowai Character Tomozakikun /  弱角友崎同学 2nd STAGE - 09 [1080P][Baha][WEB-DL][AAC AVC][CHT][MP4]"#,
            r#"{
                    "name": "Yowai Character Tomozakikun /  弱角友崎同学 2nd STAGE",
                    "season": 2,
                    "season_raw": "2nd",
                    "episode_index": 9,
                    "subtitle": "CHT",
                    "source": "Baha",
                    "fansub": "ANi",
                    "resolution": "1080P"
                }"#,
        )
    }

    #[rstest]
    #[test]
    fn test_parse_ep_with_season_en_and_season_zh(before_each: ()) -> Result<()> {
        test_parse_origin_data(
            r#"[豌豆字幕组&LoliHouse] 王者天下 第五季 / Kingdom S5 - 07 [WebRip 1080p HEVC-10bit AAC][简繁外挂字幕]"#,
            r#"{
                    "name": "王者天下 第五季 / Kingdom S5",
                    "season": 5,
                    "season_raw": "第五季",
                    "episode_index": 7,
                    "subtitle": "简繁外挂字幕",
                    "source": "WebRip",
                    "fansub": "豌豆字幕组&LoliHouse",
                    "resolution": "1080p"
                }"#,
        )
    }

    #[rstest]
    #[test]
    fn test_parse_ep_with_airota_fansub_style_case1(before_each: ()) -> Result<()> {
        test_parse_origin_data(
            r#"【千夏字幕组】【爱丽丝与特蕾丝的虚幻工厂_Alice to Therese no Maboroshi Koujou】[剧场版][WebRip_1080p_HEVC][简繁内封][招募新人]"#,
            r#"{
                  "name": "爱丽丝与特蕾丝的虚幻工厂_Alice to Therese no Maboroshi Koujou 剧场版",
                  "season": 1,
                  "episode_index": 1,
                  "subtitle": "简繁内封",
                  "source": "WebRip",
                  "fansub": "千夏字幕组",
                  "resolution": "1080p"
                }"#,
        )
    }

    #[rstest]
    #[test]
    fn test_parse_ep_with_airota_fansub_style_case2(before_each: ()) -> Result<()> {
        test_parse_origin_data(
            r#"[千夏字幕组&喵萌奶茶屋][电影 轻旅轻营 (摇曳露营) _Yuru Camp Movie][剧场版][UHDRip_2160p_HEVC][繁体][千夏15周年]"#,
            r#"{
                      "name": "电影 轻旅轻营 (摇曳露营) _Yuru Camp Movie 剧场版",
                      "season": 1,
                      "episode_index": 1,
                      "subtitle": "繁体",
                      "source": "UHDRip",
                      "fansub": "千夏字幕组&喵萌奶茶屋",
                      "resolution": "2160p"
                }"#,
        )
    }

    #[rstest]
    #[test]
    fn test_parse_ep_with_large_episode_style(before_each: ()) -> Result<()> {
        test_parse_origin_data(
            r#"[梦蓝字幕组]New Doraemon 哆啦A梦新番[747][2023.02.25][AVC][1080P][GB_JP][MP4]"#,
            r#"{
                      "name": "New Doraemon 哆啦A梦新番",
                      "season": 1,
                      "episode_index": 747,
                      "subtitle": "GB",
                      "fansub": "梦蓝字幕组",
                      "resolution": "1080P"
                    }"#,
        )
    }

    #[rstest]
    #[test]
    fn test_parse_ep_with_many_square_brackets_split_title(before_each: ()) -> Result<()> {
        test_parse_origin_data(
            r#"【MCE汉化组】[剧场版-摇曳露营][Yuru Camp][Movie][简日双语][1080P][x264 AAC]"#,
            r#"{
                  "name": "剧场版-摇曳露营 Yuru Camp Movie",
                  "season": 1,
                  "episode_index": 1,
                  "subtitle": "简日双语",
                  "fansub": "MCE汉化组",
                  "resolution": "1080P"
                }"#,
        )
    }

    #[rstest]
    #[test]
    fn test_parse_ep_with_implicit_lang_title_sep(before_each: ()) -> Result<()> {
        test_parse_origin_data(
            r#"[织梦字幕组][尼尔：机械纪元 NieR Automata Ver1.1a][02集][1080P][AVC][简日双语]"#,
            r#"{
                      "name": "尼尔：机械纪元 NieR Automata Ver1.1a",
                      "season": 1,
                      "episode_index": 2,
                      "subtitle": "简日双语",
                      "fansub": "织梦字幕组",
                      "resolution": "1080P"
                    }"#,
        )
    }

    #[rstest]
    #[test]
    fn test_parse_ep_with_square_brackets_wrapped_and_space_split(
        before_each: (),
    ) -> Result<()> {
        test_parse_origin_data(
            r#"[天月搬运组][迷宫饭 Delicious in Dungeon][03][日语中字][MKV][1080P][NETFLIX][高画质版]"#,
            r#"
                {
                  "name": "迷宫饭 Delicious in Dungeon",
                  "season": 1,
                  "episode_index": 3,
                  "subtitle": "日语中字",
                  "source": "NETFLIX",
                  "fansub": "天月搬运组",
                  "resolution": "1080P"
                }
                "#,
        )
    }

    #[rstest]
    #[test]
    fn test_parse_ep_with_start_with_brackets_wrapped_season_info_prefix(
        before_each: (),
    ) -> Result<()> {
        test_parse_origin_data(
            r#"[爱恋字幕社][1月新番][迷宫饭][Dungeon Meshi][01][1080P][MP4][简日双语] "#,
            r#"{
                  "name": "迷宫饭 Dungeon Meshi",
                  "season": 1,
                  "episode_index": 1,
                  "subtitle": "简日双语",
                  "fansub": "爱恋字幕社",
                  "resolution": "1080P"
                }"#,
        )
    }

    #[rstest]
    #[test]
    fn test_parse_ep_with_small_no_title_extra_brackets_case(
        before_each: (),
    ) -> Result<()> {
        test_parse_origin_data(
            r#"[ANi] Mahou Shoujo ni Akogarete / 梦想成为魔法少女 [年龄限制版] - 09 [1080P][Baha][WEB-DL][AAC AVC][CHT][MP4]"#,
            r#"{
                  "name": "Mahou Shoujo ni Akogarete / 梦想成为魔法少女 年龄限制版",
                  "season": 1,
                  "episode_index": 9,
                  "subtitle": "CHT",
                  "source": "Baha",
                  "fansub": "ANi",
                  "resolution": "1080P"
                }"#,
        )
    }

    #[rstest]
    #[test]
    fn test_parse_ep_title_leading_space_style(before_each: ()) -> Result<()> {
        test_parse_origin_data(
            r#"[ANi]  16bit 的感动 ANOTHER LAYER - 01 [1080P][Baha][WEB-DL][AAC AVC][CHT][MP4]"#,
            r#"{
                "name": "16bit 的感动 ANOTHER LAYER",
                "season": 1,
                "episode_index": 1,
                "subtitle": "CHT",
                "source": "Baha",
                "fansub": "ANi",
                "resolution": "1080P"
            }"#,
        )
    }

    #[rstest]
    #[test]
    fn test_parse_ep_title_leading_month_and_wrapped_brackets_style(
        before_each: (),
    ) -> Result<()> {
        test_parse_origin_data(
            r#"【喵萌奶茶屋】★07月新番★[银砂糖师与黑妖精 ~ Sugar Apple Fairy Tale ~][13][1080p][简日双语][招募翻译]"#,
            r#"{
                "name": "银砂糖师与黑妖精 ~ Sugar Apple Fairy Tale ~",
                "season": 1,
                "episode_index": 13,
                "subtitle": "简日双语",
                "fansub": "喵萌奶茶屋",
                "resolution": "1080p"
            }"#,
        )
    }

    #[rstest]
    #[test]
    fn test_parse_ep_title_leading_month_style(before_each: ()) -> Result<()> {
        test_parse_origin_data(
            r#"【极影字幕社】★4月新番 天国大魔境 Tengoku Daimakyou 第05话 GB 720P MP4（字幕社招人内详）"#,
            r#"{
                "name": "天国大魔境 Tengoku Daimakyou",
                "season": 1,
                "episode_index": 5,
                "subtitle": "GB",
                "fansub": "极影字幕社",
                "resolution": "720P"
            }"#,
        )
    }

    #[rstest]
    #[test]
    fn test_parse_ep_tokusatsu_style(before_each: ()) -> Result<()> {
        test_parse_origin_data(
            r#"[MagicStar] 假面骑士Geats / 仮面ライダーギーツ EP33 [WEBDL] [1080p] [TTFC]【生】"#,
            r#"{
              "name": "假面骑士Geats / 仮面ライダーギーツ",
              "season": 1,
              "episode_index": 33,
              "source": "WEBDL",
              "subtitle": "生",
              "fansub": "MagicStar",
              "resolution": "1080p"
            }"#,
        )
    }

    #[rstest]
    #[test]
    fn test_parse_ep_with_multi_lang_zh_title(before_each: ()) -> Result<()> {
        test_parse_origin_data(
            r#"[百冬练习组&LoliHouse] BanG Dream! 少女乐团派对！☆PICO FEVER！ / Garupa Pico: Fever! - 26 [WebRip 1080p HEVC-10bit AAC][简繁内封字幕][END] [101.69 MB]"#,
            r#"{
                "name": "BanG Dream! 少女乐团派对！☆PICO FEVER！ / Garupa Pico: Fever!",
                "season": 1,
                "episode_index": 26,
                "subtitle": "简繁内封字幕",
                "source": "WebRip",
                "fansub": "百冬练习组&LoliHouse",
                "resolution": "1080p"
            }"#,
        )
    }

    #[rstest]
    #[test]
    fn test_ep_collections(before_each: ()) -> Result<()> {
        test_parse_origin_data(
            r#"[奶²&LoliHouse] 蘑菇狗 / Kinokoinu: Mushroom Pup [01-12 精校合集][WebRip 1080p HEVC-10bit AAC][简日内封字幕]"#,
            r#"{
                "name": "蘑菇狗 / Kinokoinu: Mushroom Pup",
                "season": 1,
                "episode_index": 1,
                "subtitle": "简日内封字幕",
                "source": "WebRip",
                "fansub": "奶²&LoliHouse",
                "resolution": "1080p"
            }"#,
        )?;

        test_parse_origin_data(
            r#"[LoliHouse] 叹气的亡灵想隐退 / Nageki no Bourei wa Intai shitai [01-13 合集][WebRip 1080p HEVC-10bit AAC][简繁内封字幕][Fin]"#,
            r#"{
                "name": "叹气的亡灵想隐退 / Nageki no Bourei wa Intai shitai",
                "season": 1,
                "episode_index": 1,
                "subtitle": "简繁内封字幕",
                "source": "WebRip",
                "fansub": "LoliHouse",
                "resolution": "1080p"
            }"#,
        )?;

        test_parse_origin_data(
            r#"[LoliHouse] 精灵幻想记 第二季 / Seirei Gensouki S2 [01-12 合集][WebRip 1080p HEVC-10bit AAC][简繁内封字幕][Fin]"#,
            r#"{
                "name": "精灵幻想记 第二季 / Seirei Gensouki S2",
                "season": 2,
                "season_raw": "第二季",
                "episode_index": 1,
                "subtitle": "简繁内封字幕",
                "source": "WebRip",
                "fansub": "LoliHouse",
                "resolution": "1080p"
            }"#,
        )?;

        test_parse_origin_data(
            r#"[喵萌奶茶屋&LoliHouse] 超自然武装当哒当 / 胆大党 / Dandadan [01-12 精校合集][WebRip 1080p HEVC-10bit AAC][简繁日内封字幕][Fin]"#,
            r#" {
                "name": "超自然武装当哒当 / 胆大党 / Dandadan",
                "season": 1,
                "episode_index": 1,
                "subtitle": "简繁日内封字幕",
                "source": "WebRip",
                "fansub": "喵萌奶茶屋&LoliHouse",
                "resolution": "1080p"
            }"#,
        )?;

        Ok(())
    }

    #[rstest]
    #[test]
    fn test_parse_ep_with_zh_bracketed_name(before_each: ()) -> Result<()> {
        test_parse_origin_data(
            r#"【幻樱字幕组】【4月新番】【古见同学有交流障碍症 第二季 Komi-san wa, Komyushou Desu. S02】【22】【GB_MP4】【1920X1080】"#,
            r#"{
                "name": "古见同学有交流障碍症 第二季 Komi-san wa, Komyushou Desu. S02",
                "season": 2,
                "season_raw": "第二季",
                "episode_index": 22,
                "subtitle": "GB",
                "fansub": "幻樱字幕组",
                "resolution": "1920X1080"
            }"#,
        )
    }

    #[rstest]
    #[test]
    fn test_bad_cases(before_each: ()) -> Result<()> {
        test_parse_origin_data(
            r#"[7³ACG x 桜都字幕组] 摇曳露营△ 剧场版/映画 ゆるキャン△/Eiga Yuru Camp△ [简繁字幕] BDrip 1080p x265 FLAC 2.0"#,
            r#"{
                  "name": "摇曳露营△ 剧场版",
                  "season": 1,
                  "episode_index": 1,
                  "subtitle": "简繁字幕",
                  "source": "BDrip",
                  "fansub": "7³ACG x 桜都字幕组",
                  "resolution": "1080p"
                }"#,
        )?;

        Ok(())
    }
}
