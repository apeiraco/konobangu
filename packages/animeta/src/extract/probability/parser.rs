use crate::extract::legacy::OriginNameMeta;

use super::compose::compose;
use super::keywords::DEFAULT_DB;
use super::resolve::{calculate_confidence, resolve};
use super::rules::possibility::apply_all_possibility_rules;
use super::rules::score::apply_all_score_rules;
use super::tokenizer::tokenize;
use super::types::*;

#[derive(Debug)]
pub struct ParseResult {
    pub meta: OriginNameMeta,
    pub confidence: f64,
}

#[derive(Debug)]
pub struct ParseInspection {
    pub meta: OriginNameMeta,
    pub confidence: f64,
    pub tokens: Vec<Token>,
}

pub fn parse(input: &str) -> ParseResult {
    let (mut tokens, groups, profile) = tokenize(input);

    apply_all_possibility_rules(&mut tokens, &groups, &profile, &DEFAULT_DB);
    apply_all_score_rules(&mut tokens, &groups);
    resolve(&mut tokens);

    let confidence = calculate_confidence(&tokens);
    let meta = compose(&tokens, input);

    ParseResult { meta, confidence }
}

pub fn parse_with_inspection(input: &str) -> ParseInspection {
    let (mut tokens, groups, profile) = tokenize(input);

    apply_all_possibility_rules(&mut tokens, &groups, &profile, &DEFAULT_DB);
    apply_all_score_rules(&mut tokens, &groups);
    resolve(&mut tokens);

    let confidence = calculate_confidence(&tokens);
    let meta = compose(&tokens, input);

    ParseInspection {
        meta,
        confidence,
        tokens,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    fn test_parse_parity(origin_name: &str, expected_json: &str) -> Result<()> {
        let result = parse(origin_name);
        let found = result.meta;
        let expected: OriginNameMeta = serde_json::from_str(expected_json).inspect_err(|e| {
            eprintln!(
                "Failed to parse expected: {}, but got found: {}",
                e,
                serde_json::to_string_pretty(&found).unwrap()
            );
        })?;

        if expected != found {
            eprintln!(
                "INPUT: {}\nexpected:\n{}\nfound:\n{}",
                origin_name,
                serde_json::to_string_pretty(&expected).unwrap(),
                serde_json::to_string_pretty(&found).unwrap()
            );
        }
        assert_eq!(expected, found);

        Ok(())
    }

    #[test]
    fn test_parse_basic() {
        let result = parse("[SubGroup] Anime Title - 01 [1080p HEVC].mkv");
        assert!(!result.meta.name.is_empty());
        assert_eq!(result.meta.season, 1);
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_parse_with_inspection() {
        let result = parse_with_inspection("[SubGroup] Title - 01 [720p]");
        assert!(!result.tokens.is_empty());
        assert!(!result.meta.name.is_empty());
    }

    #[test]
    fn test_parse_ep_with_all_parts_wrapped() -> Result<()> {
        test_parse_parity(
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

    #[test]
    fn test_parse_ep_with_title_wrapped_by_one_square_bracket_and_season_prefix() -> Result<()> {
        test_parse_parity(
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

    #[test]
    fn test_parse_ep_with_ep_and_version() -> Result<()> {
        test_parse_parity(
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

    #[test]
    fn test_parse_ep_with_en_title_only() -> Result<()> {
        test_parse_parity(
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

    #[test]
    fn test_parse_ep_with_two_zh_title() -> Result<()> {
        test_parse_parity(
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

    #[test]
    fn test_parse_ep_with_en_zh_jp_titles() -> Result<()> {
        test_parse_parity(
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

    #[test]
    fn test_parse_ep_with_nth_season() -> Result<()> {
        test_parse_parity(
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

    #[test]
    fn test_parse_ep_with_season_en_and_season_zh() -> Result<()> {
        test_parse_parity(
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

    #[test]
    fn test_parse_ep_with_airota_fansub_style_case1() -> Result<()> {
        test_parse_parity(
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

    #[test]
    fn test_parse_ep_with_airota_fansub_style_case2() -> Result<()> {
        test_parse_parity(
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

    #[test]
    fn test_parse_ep_with_large_episode_style() -> Result<()> {
        test_parse_parity(
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

    #[test]
    fn test_parse_ep_with_many_square_brackets_split_title() -> Result<()> {
        test_parse_parity(
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

    #[test]
    fn test_parse_ep_with_implicit_lang_title_sep() -> Result<()> {
        test_parse_parity(
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

    #[test]
    fn test_parse_ep_with_square_brackets_wrapped_and_space_split() -> Result<()> {
        test_parse_parity(
            r#"[天月搬运组][迷宫饭 Delicious in Dungeon][03][日语中字][MKV][1080P][NETFLIX][高画质版]"#,
            r#"{
                "name": "迷宫饭 Delicious in Dungeon",
                "season": 1,
                "episode_index": 3,
                "subtitle": "日语中字",
                "source": "NETFLIX",
                "fansub": "天月搬运组",
                "resolution": "1080P"
            }"#,
        )
    }

    #[test]
    fn test_parse_ep_with_start_with_brackets_wrapped_season_info_prefix() -> Result<()> {
        test_parse_parity(
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

    #[test]
    fn test_parse_ep_with_small_no_title_extra_brackets_case() -> Result<()> {
        test_parse_parity(
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

    #[test]
    fn test_parse_ep_title_leading_space_style() -> Result<()> {
        test_parse_parity(
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

    #[test]
    fn test_parse_ep_title_leading_month_and_wrapped_brackets_style() -> Result<()> {
        test_parse_parity(
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

    #[test]
    fn test_parse_ep_title_leading_month_style() -> Result<()> {
        test_parse_parity(
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

    #[test]
    fn test_parse_ep_tokusatsu_style() -> Result<()> {
        test_parse_parity(
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

    #[test]
    fn test_parse_ep_with_multi_lang_zh_title() -> Result<()> {
        test_parse_parity(
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

    #[test]
    fn test_ep_collections() -> Result<()> {
        test_parse_parity(
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

        test_parse_parity(
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

        test_parse_parity(
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

        test_parse_parity(
            r#"[喵萌奶茶屋&LoliHouse] 超自然武装当哒当 / 胆大党 / Dandadan [01-12 精校合集][WebRip 1080p HEVC-10bit AAC][简繁日内封字幕][Fin]"#,
            r#"{
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

    #[test]
    fn test_parse_ep_with_zh_bracketed_name() -> Result<()> {
        test_parse_parity(
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

    #[test]
    fn test_bad_cases() -> Result<()> {
        test_parse_parity(
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
