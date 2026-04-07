use std::collections::HashMap;

use icu::properties::{CodePointMapData, props::Script};
use lazy_static::lazy_static;
use maplit::hashmap;
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, digit1, none_of, satisfy},
    combinator::{map, opt, recognize, value, verify},
    error::ParseError,
    multi::many1,
    sequence::{delimited, preceded},
};
use num_traits::{PrimInt, Signed};

lazy_static! {
    pub static ref ZH_DIGIT_MAP: HashMap<char, u32> = {
        hashmap! {
            '〇' => 0,
            '零' => 0,
            '一' => 1,
            '壹' => 1,
            '二' => 2,
            '贰' => 2,
            '三' => 3,
            '叁' => 3,
            '四' => 4,
            '肆' => 4,
            '五' => 5,
            '伍' => 5,
            '六' => 6,
            '陆' => 6,
            '七' => 7,
            '柒' => 7,
            '八' => 8,
            '捌' => 8,
            '九' => 9,
            '玖' => 9,
            '十' => 10,
            '拾' => 10,
            '廿' => 20,
            '念' => 20,
            '百' => 100,
            '佰' => 100,
            '千' => 1000,
            '仟' => 1000,
            '万' => 10000,
            '萬' => 10000,
            '亿' => 100000000,
            '億' => 100000000,
        }
    };
}

pub fn with_recognized<'a, F, O, E>(
    mut parser: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, (O, &'a str), E>
where
    F: Parser<&'a str, Output = O, Error = E>,
    E: ParseError<&'a str>,
{
    move |input: &'a str| {
        let i = input;
        let (rest, output) = parser.parse(i)?;
        let consumed_len = i.len() - rest.len();
        Ok((rest, (output, &i[..consumed_len])))
    }
}

pub fn is_some_unicode_scx(input: &str, script: Script) -> IResult<&str, char> {
    let script_data = CodePointMapData::<Script>::new();

    verify(anychar, |&c| script_data.get(c) == script).parse(input)
}

pub fn is_han_scx(input: &str) -> IResult<&str, char> {
    is_some_unicode_scx(input, Script::Han)
}

pub fn is_hira_scx(input: &str) -> IResult<&str, char> {
    is_some_unicode_scx(input, Script::Hiragana)
}

pub fn is_kana_scx(input: &str) -> IResult<&str, char> {
    is_some_unicode_scx(input, Script::Katakana)
}

pub fn delimited_by_brackets(input: &str) -> IResult<&str, &str> {
    alt((
        delimited(tag("["), recognize(many1(none_of("[]"))), tag("]")),
        delimited(tag("【"), recognize(many1(none_of("【】"))), tag("】")),
    ))
    .parse(input)
}

pub struct ZhNum {
    pub int: i32,
}

impl ZhNum {
    fn parse_digit<'a>(
        max_value: u32,
    ) -> impl Parser<&'a str, Output = u32, Error = nom::error::Error<&'a str>> {
        map(
            satisfy(move |c| ZH_DIGIT_MAP.get(&c).is_some_and(|v| *v <= max_value)),
            |c| *ZH_DIGIT_MAP.get(&c).unwrap(),
        )
    }

    fn parse_个(input: &str) -> IResult<&str, u32> {
        Self::parse_digit(9).parse(input)
    }

    fn parse_十(input: &str) -> IResult<&str, u32> {
        let (input, (p, o, s)) = (
            opt(Self::parse_个),
            map(
                satisfy(|c| ZH_DIGIT_MAP.get(&c).is_some_and(|v| *v == 10 || *v == 20)),
                |c| *ZH_DIGIT_MAP.get(&c).unwrap(),
            ),
            opt(Self::parse_个),
        )
            .parse(input)?;

        let value = p.unwrap_or(1) * o + s.unwrap_or(0);

        Ok((input, value))
    }

    pub fn parse_百(input: &str) -> IResult<&str, u32> {
        let (input, (p, o, s)) = (
            opt(Self::parse_个),
            map(
                satisfy(|c| ZH_DIGIT_MAP.get(&c).is_some_and(|v| *v == 100 || *v == 200)),
                |c| *ZH_DIGIT_MAP.get(&c).unwrap(),
            ),
            opt(Self::parse_十),
        )
            .parse(input)?;

        let value = p.unwrap_or(1) * o + s.unwrap_or(0);

        Ok((input, value))
    }

    pub fn parse_千(input: &str) -> IResult<&str, u32> {
        let (input, (p, o, s)) = (
            opt(Self::parse_个),
            value(
                1000u32,
                satisfy(|c| ZH_DIGIT_MAP.get(&c).is_some_and(|v| *v == 1000)),
            ),
            opt(Self::parse_百),
        )
            .parse(input)?;

        let value = p.unwrap_or(1) * o + s.unwrap_or(0);

        Ok((input, value))
    }

    pub fn parse_万(input: &str) -> IResult<&str, u32> {
        let (input, (p, o, s)) = (
            opt(Self::parse_千),
            value(
                10000u32,
                satisfy(|c| ZH_DIGIT_MAP.get(&c).is_some_and(|v| *v == 10000)),
            ),
            opt(Self::parse_千),
        )
            .parse(input)?;

        let value = p.unwrap_or(1) * o + s.unwrap_or(0);

        Ok((input, value))
    }

    pub fn parse_亿(input: &str) -> IResult<&str, u32> {
        let (input, (p, o, s)) = (
            opt(Self::parse_万),
            value(
                100000000u32,
                satisfy(|c| ZH_DIGIT_MAP.get(&c).is_some_and(|v| *v == 100000000)),
            ),
            opt(Self::parse_万),
        )
            .parse(input)?;

        let value = p.unwrap_or(1) * o + s.unwrap_or(0);

        Ok((input, value))
    }

    pub fn parse_uint(input: &str) -> IResult<&str, u32> {
        preceded(
            opt(tag("正")),
            alt((
                Self::parse_个,
                Self::parse_十,
                Self::parse_百,
                Self::parse_千,
                Self::parse_万,
                Self::parse_亿,
            )),
        )
        .parse(input)
    }

    pub fn parse_int(input: &str) -> IResult<&str, i32> {
        let (input, (sign, value)) = (
            opt(alt((value(1, tag("正")), value(-1, tag("负"))))),
            alt((
                Self::parse_个,
                Self::parse_十,
                Self::parse_百,
                Self::parse_千,
                Self::parse_万,
                Self::parse_亿,
            )),
        )
            .parse(input)?;

        Ok((input, sign.unwrap_or(1) * value as i32))
    }
}

pub fn parse_uint<T: PrimInt>(input: &str) -> IResult<&str, T> {
    let (input, value) = preceded(opt(tag("+")), digit1).parse(input)?;

    let value = T::from_str_radix(value, 10).map_err(|_| {
        nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Digit))
    })?;

    Ok((input, value))
}

pub fn parse_int<T: PrimInt + Signed>(input: &str) -> IResult<&str, T> {
    let (input, value) = recognize((
        opt(alt((
            value(T::one(), tag("+")),
            value(T::one().neg(), tag("-")),
        ))),
        digit1,
    ))
    .parse(input)?;

    let value = T::from_str_radix(value, 10).map_err(|_| {
        nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Digit))
    })?;

    Ok((input, value))
}

pub fn parse_month_num(input: &str) -> IResult<&str, u32> {
    verify(alt((ZhNum::parse_uint, parse_uint::<u32>)), |v| {
        *v <= 12 && *v > 0
    })
    .parse(input)
}
