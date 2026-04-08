use std::sync::LazyLock;

use regex::Regex;

use super::{tokenizer::*, types::*};
use crate::extract::legacy::OriginNameMeta;

static ORDINAL_SEASON: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)(\d{1,2})(?:st|nd|rd|th)").unwrap());

pub fn compose(tokens: &[Token], input: &str) -> OriginNameMeta {
  let title = extract_title(tokens, input);
  let (season, season_raw) = extract_season(tokens);
  let episode_index = extract_episode(tokens);
  let fansub = extract_fansub(tokens, input);
  let source = extract_source(tokens, input);
  let subtitle = extract_subtitle(tokens, input);
  let resolution = extract_resolution(tokens, input);

  OriginNameMeta {
    name: title,
    season,
    season_raw,
    episode_index,
    subtitle,
    source,
    fansub,
    resolution,
  }
}

fn extract_title(tokens: &[Token], input: &str) -> String {
  let mut title_spans: Vec<(usize, usize)> = Vec::new();

  for token in tokens {
    if token.resolved == Some(Label::Title) {
      title_spans.push((token.span.start, token.span.end));
    }
  }

  if title_spans.is_empty() {
    // Fallback: use all unknown tokens outside brackets
    for token in tokens {
      if token.resolved == Some(Label::Unknown) && token.bracket_group.is_none() {
        title_spans.push((token.span.start, token.span.end));
      }
    }
  }

  if title_spans.is_empty() {
    return String::new();
  }

  // Find contiguous title regions and join them
  let mut result_parts: Vec<&str> = Vec::new();
  let mut current_start = title_spans[0].0;
  let mut current_end = title_spans[0].1;

  for &(start, end) in &title_spans[1..] {
    // Check if this span is adjacent or close to current (within a small gap)
    let gap = &input[current_end..start];
    let gap_is_delimiter = gap.chars().all(|c| {
      DELIMITERS.contains(c)
        || OPEN_BRACKETS.contains(&c)
        || CLOSE_BRACKETS.contains(&c)
        || c == '/'
        || matches!(c, '!' | '?' | '！' | '？' | '：' | '・' | '☆' | '，' | ',' | '、')
    });

    if gap_is_delimiter && gap.len() <= 24 {
      current_end = end;
    } else {
      result_parts.push(&input[current_start..current_end]);
      current_start = start;
      current_end = end;
    }
  }
  result_parts.push(&input[current_start..current_end]);

  let raw = result_parts.join(" ");
  clean_title(&raw)
}

fn clean_title(raw: &str) -> String {
  let trimmed = raw.trim();
  // Legacy parity: primary title before alternate (slash-separated) titles
  let trimmed = trimmed.split('/').next().unwrap_or(trimmed).trim();
  // Remove leading/trailing delimiters and brackets
  let cleaned: String = trimmed
    .trim_matches(|c: char| DELIMITERS.contains(c) || OPEN_BRACKETS.contains(&c) || CLOSE_BRACKETS.contains(&c))
    .to_string();

  // Collapse multiple whitespace
  let mut result = String::with_capacity(cleaned.len());
  let mut prev_space = false;
  for c in cleaned.chars() {
    if c.is_whitespace() {
      if !prev_space {
        result.push(' ');
      }
      prev_space = true;
    } else {
      result.push(c);
      prev_space = false;
    }
  }
  result.trim().to_string()
}

fn extract_season(tokens: &[Token]) -> (i32, Option<String>) {
  // Prefer Chinese season in title (第五季) over English S5 when both exist
  for token in tokens {
    if token.resolved == Some(Label::Title)
      && let Some(n) = parse_zh_season(&token.content)
    {
      return (n, Some(token.content.clone()));
    }
  }

  // Look for tokens resolved as SeasonPrefix followed by SequenceNumber
  for (i, token) in tokens.iter().enumerate() {
    if token.resolved == Some(Label::SeasonPrefix) {
      // Find next sequence number
      for next in &tokens[i + 1..] {
        if next.resolved == Some(Label::Delimiter) {
          continue;
        }
        if next.resolved == Some(Label::SequenceNumber)
          && let Ok(n) = next.content.parse::<i32>()
        {
          let raw = format!("{}{}", token.content, next.content);
          return (n, Some(raw));
        }
        break;
      }
    }
  }

  // Check title tokens for inline season indicators like "S2", "第二季", "2nd"
  let title_text: String = tokens
    .iter()
    .filter(|t| t.resolved == Some(Label::Title))
    .map(|t| t.content.as_str())
    .collect::<Vec<_>>()
    .join("");

  if let Some(n) = parse_season_from_text(&title_text) {
    let raw = extract_season_raw_from_title(tokens, &title_text);
    return (n, raw);
  }

  let broad_title: String = tokens
    .iter()
    .filter(|t| t.zone == Zone::Title && matches!(t.resolved, Some(Label::Title | Label::Unknown)))
    .map(|t| t.content.as_str())
    .collect::<Vec<_>>()
    .join("");

  if let Some(n) = parse_season_from_text(&broad_title) {
    let raw = extract_season_raw_from_title(tokens, &broad_title);
    return (n, raw);
  }

  (1, None)
}

fn parse_zh_season(text: &str) -> Option<i32> {
  // Match patterns like 第二季, 第2季, 第二期
  let text = text.trim();
  let rest = text.strip_prefix('第')?;

  let num_str = if rest.ends_with('季') {
    &rest[..rest.len() - '季'.len_utf8()]
  } else if rest.ends_with('期') {
    &rest[..rest.len() - '期'.len_utf8()]
  } else {
    return None;
  };

  // Try numeric
  if let Ok(n) = num_str.parse::<i32>() {
    return Some(n);
  }

  // Try Chinese numerals
  parse_zh_numeral(num_str)
}

fn parse_zh_numeral(s: &str) -> Option<i32> {
  let s = s.trim();
  match s {
    "一" => Some(1),
    "二" => Some(2),
    "三" => Some(3),
    "四" => Some(4),
    "五" => Some(5),
    "六" => Some(6),
    "七" => Some(7),
    "八" => Some(8),
    "九" => Some(9),
    "十" => Some(10),
    "十一" => Some(11),
    "十二" => Some(12),
    _ => None,
  }
}

fn parse_season_from_text(text: &str) -> Option<i32> {
  let upper = text.to_uppercase();
  // S2, S02
  if let Some(rest) = upper.strip_prefix('S')
    && let Ok(n) = rest.trim().parse::<i32>()
  {
    return Some(n);
  }
  if let Some(cap) = ORDINAL_SEASON.captures(text)
    && let Ok(n) = cap[1].parse::<i32>()
  {
    return Some(n);
  }
  if let Some(n) = parse_zh_season(text) {
    return Some(n);
  }
  None
}

fn extract_season_raw_from_title(tokens: &[Token], broad_title: &str) -> Option<String> {
  for token in tokens {
    if token.resolved != Some(Label::Title) {
      continue;
    }
    if parse_zh_season(&token.content).is_some() {
      return Some(token.content.clone());
    }
    let upper = token.content.to_uppercase();
    if upper.starts_with('S') && upper[1..].parse::<i32>().is_ok() {
      return Some(token.content.clone());
    }
  }
  ORDINAL_SEASON.find(broad_title).map(|m| m.as_str().to_string())
}

fn find_prev_non_delimiter(tokens: &[Token], idx: usize) -> Option<usize> {
  (0..idx).rev().find(|&i| tokens[i].class != TokenClass::Delimiter)
}

fn is_alone_non_delim_in_bracket(tokens: &[Token], idx: usize) -> bool {
  let bg = match tokens[idx].bracket_group {
    Some(bg) => bg,
    None => return false,
  };
  tokens
    .iter()
    .filter(|t| t.bracket_group == Some(bg) && t.class != TokenClass::BracketOpen && t.class != TokenClass::BracketClose && t.class != TokenClass::Delimiter)
    .count()
    == 1
}

fn extract_episode(tokens: &[Token]) -> i32 {
  // Look for episode prefix + number (第05集, EP12, ...)
  for (i, token) in tokens.iter().enumerate() {
    if token.resolved == Some(Label::EpisodePrefix) {
      for next in &tokens[i + 1..] {
        if next.resolved == Some(Label::Delimiter) {
          continue;
        }
        if next.resolved == Some(Label::SequenceNumber)
          && let Ok(n) = next.content.parse::<i32>()
        {
          return n;
        }
        break;
      }
    }
  }

  // Prefer a lone episode index in its own bracket: [05], [747]
  for (i, token) in tokens.iter().enumerate() {
    if token.resolved != Some(Label::SequenceNumber) {
      continue;
    }
    if let Ok(n) = token.content.parse::<i32>()
      && (0..=9999).contains(&n)
      && is_alone_non_delim_in_bracket(tokens, i)
    {
      return n;
    }
  }

  // After dash: "Title - 08"
  for (i, token) in tokens.iter().enumerate() {
    if token.resolved != Some(Label::SequenceNumber) {
      continue;
    }
    if let Ok(n) = token.content.parse::<i32>() {
      if !(0..=9999).contains(&n) {
        continue;
      }
      if let Some(p) = find_prev_non_delimiter(tokens, i)
        && tokens[p].resolved == Some(Label::ContextDelimiter)
      {
        return n;
      }
    }
  }

  let mut best_episode: Option<i32> = None;
  for (i, token) in tokens.iter().enumerate() {
    if token.resolved != Some(Label::SequenceNumber) {
      continue;
    }
    if let Some(p) = find_prev_non_delimiter(tokens, i)
      && tokens[p].possibilities.contains_key(&Label::SeasonPrefix)
    {
      continue;
    }
    if let Ok(n) = token.content.parse::<i32>()
      && (0..=9999).contains(&n)
    {
      best_episode = Some(n);
      break;
    }
  }

  let has_movie = tokens.iter().any(|t| t.resolved == Some(Label::SeriesType));
  if best_episode.is_none() && has_movie {
    return 1;
  }

  best_episode.unwrap_or(1)
}

fn extract_fansub(tokens: &[Token], input: &str) -> Option<String> {
  // Fansub is typically in the first bracket group resolved as ReleaseGroup
  let first_bg = tokens.iter().position(|t| t.class == TokenClass::BracketOpen)?;

  let close = tokens[first_bg + 1..]
    .iter()
    .position(|t| t.class == TokenClass::BracketClose)
    .map(|p| p + first_bg + 1)?;

  let inner = &tokens[first_bg + 1..close];

  // Check if most tokens inside are resolved as ReleaseGroup
  let rg_count = inner.iter().filter(|t| t.resolved == Some(Label::ReleaseGroup)).count();
  let non_delim_count = inner.iter().filter(|t| t.class != TokenClass::Delimiter).count();

  if rg_count > 0 && rg_count * 2 >= non_delim_count {
    let start = inner.first()?.span.start;
    let end = inner.last()?.span.end;
    let text = &input[start..end];
    return Some(text.trim().to_string());
  }

  None
}

fn extract_source(tokens: &[Token], input: &str) -> Option<String> {
  let source_idxs: Vec<usize> = tokens
    .iter()
    .enumerate()
    .filter(|(_, t)| t.resolved == Some(Label::Source))
    .map(|(i, _)| i)
    .collect();
  if source_idxs.is_empty() {
    return None;
  }

  let start = tokens[source_idxs[0]].span.start;
  let mut end = tokens[source_idxs[0]].span.end;
  for w in source_idxs.windows(2) {
    let gap = &tokens[w[0] + 1..w[1]];
    let gap_ok = gap
      .iter()
      .all(|t| t.class == TokenClass::Delimiter || t.content == "_" || t.content == "-" || t.content == ".");
    if gap_ok {
      end = end.max(tokens[w[1]].span.end);
    } else {
      break;
    }
  }
  Some(input[start..end].to_string())
}

fn extract_subtitle(tokens: &[Token], input: &str) -> Option<String> {
  // Find bracket groups containing language/subtitle tokens
  let mut best_subtitle: Option<String> = None;

  let mut i = 0;
  while i < tokens.len() {
    if tokens[i].class == TokenClass::BracketOpen {
      let bg = tokens[i].bracket_group;
      // Find close bracket
      let close = tokens[i + 1..]
        .iter()
        .position(|t| t.class == TokenClass::BracketClose && t.bracket_group == bg)
        .map(|p| p + i + 1);

      if let Some(close_idx) = close {
        let inner = &tokens[i + 1..close_idx];
        let has_subtitle = inner.iter().any(|t| t.resolved == Some(Label::Language) || t.resolved == Some(Label::SubsTerm));

        if has_subtitle {
          let start = inner.first().map(|t| t.span.start).unwrap_or(0);
          let end = inner.last().map(|t| t.span.end).unwrap_or(0);
          if start < end && end <= input.len() {
            let mut text = input[start..end].trim().to_string();
            if text == "GB_JP" || (text.starts_with("GB_") && text.len() > 3) {
              text = "GB".to_string();
            }
            if !text.is_empty() {
              best_subtitle = Some(text);
            }
          }
        }
        i = close_idx + 1;
        continue;
      }
    }
    i += 1;
  }

  // Also check for language tokens outside brackets
  if best_subtitle.is_none() {
    for token in tokens {
      if (token.resolved == Some(Label::Language) || token.resolved == Some(Label::SubsTerm)) && token.bracket_group.is_none() {
        return Some(token.content.clone());
      }
    }
  }

  best_subtitle
}

static RESOLUTION_WXH: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\d{3,4}[Xx×]\d{3,4}").unwrap());
static RESOLUTION_STD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\d{3,4}[PpIi]|\d[Kk]").unwrap());

fn extract_resolution(tokens: &[Token], input: &str) -> Option<String> {
  if let Some(m) = RESOLUTION_WXH.find(input) {
    return Some(m.as_str().to_string());
  }
  if let Some(m) = RESOLUTION_STD.find(input) {
    return Some(m.as_str().to_string());
  }
  let spans: Vec<(usize, usize)> = tokens
    .iter()
    .filter(|t| t.resolved == Some(Label::VideoResolution))
    .map(|t| (t.span.start, t.span.end))
    .collect();
  if spans.is_empty() {
    return None;
  }
  let mut cur_s = spans[0].0;
  let mut cur_e = spans[0].1;
  let mut merged: Vec<(usize, usize)> = Vec::new();
  for &(s, e) in &spans[1..] {
    if s <= cur_e + 1 {
      cur_e = cur_e.max(e);
    } else {
      merged.push((cur_s, cur_e));
      cur_s = s;
      cur_e = e;
    }
  }
  merged.push((cur_s, cur_e));
  merged.into_iter().max_by_key(|(a, b)| b - a).map(|(a, b)| input[a..b].to_string())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::extract::probability::{
    keywords::DEFAULT_DB,
    resolve::resolve,
    rules::{possibility::apply_all_possibility_rules, score::apply_all_score_rules},
    tokenizer::tokenize,
  };

  fn run_parse(input: &str) -> OriginNameMeta {
    let (mut tokens, groups, profile) = tokenize(input);
    apply_all_possibility_rules(&mut tokens, &groups, &profile, &DEFAULT_DB);
    apply_all_score_rules(&mut tokens, &groups);
    resolve(&mut tokens);
    compose(&tokens, input)
  }

  #[test]
  fn test_compose_basic() {
    let meta = run_parse("[SubGroup] Anime Title - 01 [1080p HEVC]");
    assert!(!meta.name.is_empty());
    assert_eq!(meta.season, 1);
  }

  #[test]
  fn test_compose_with_fansub() {
    let meta = run_parse("[SubGroup] Anime Title - 01 [1080p]");
    assert!(meta.fansub.is_some());
  }

  #[test]
  fn test_compose_resolution() {
    let meta = run_parse("[Group] Title - 01 [1080p]");
    assert!(meta.resolution.is_some());
  }
}
