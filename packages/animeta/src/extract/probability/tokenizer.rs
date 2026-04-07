use std::collections::HashMap;

use fancy_regex::Regex;
use lazy_static::lazy_static;

use super::types::*;

pub const OPEN_BRACKETS: &[char] = &['(', '[', '{', '\u{3010}', '\u{300C}', '\u{300E}', '\u{FF08}', '\u{3014}'];
pub const CLOSE_BRACKETS: &[char] = &[')', ']', '}', '\u{3011}', '\u{300D}', '\u{300F}', '\u{FF09}', '\u{3015}'];
pub const BRACKET_PAIRS: &[(char, char)] = &[
  ('(', ')'),
  ('[', ']'),
  ('{', '}'),
  ('\u{3010}', '\u{3011}'), // 【 】
  ('\u{300C}', '\u{300D}'), // 「 」
  ('\u{300E}', '\u{300F}'), // 『 』
  ('\u{FF08}', '\u{FF09}'), // （ ）
  ('\u{3014}', '\u{3015}'), // 〔 〕
];
pub const DASHES: &str = "-\u{2010}\u{2011}\u{2012}\u{2013}\u{2014}\u{2015}\u{2212}\u{FE58}\u{FE63}\u{FF0D}";
pub const DELIMITERS: &str = " ._+,-\u{2010}\u{2011}\u{2012}\u{2013}\u{2014}\u{2015}\u{2212}\u{FE58}\u{FE63}\u{FF0D}";

lazy_static! {
    // CJK runs must NOT include full-width bracket codepoints (U+3000–303F block),
    // or closers like U+3011 get glued to the previous Han word and break pairing.
    static ref MIXED_TOKEN_REGEX: Regex = Regex::new(concat!(
        r"(?:",
        r"\d+(?:st|nd|rd|th)",                              // ordinals
        // Bit depth: exclude "-10bit" inside "HEVC-10bit" (fancy-regex: lookbehind)
        r"|(?<![-\d])\d{1,3}(?:[Bb]it|[Bb]its)",
        // Full-width / CJK brackets as their own tokens (before CJK runs)
        r"|[\u3010\u3011\u300C\u300D\u300E\u300F\uFF08\uFF09\u3014\u3015]",
        r"|[\u3000-\u300B\u3012-\u3013\u3016-\u303F\u3040-\u309F\u30A0-\u30FF",
        r"\u4E00-\u9FFF\uF900-\uFAFF]+",                   // CJK sequences (no bracket glue)
        r"|[a-z][A-Z]+",                                    // camelCase split point
        r"|[A-Z]?[a-z]+",                                   // lowercase word (optionally capitalized)
        r"|[A-Z]+",                                         // uppercase sequence
        r"|\d+",                                            // digits
        r"|_",                                              // underscore (separate since \w includes it)
        r"|[^\w\u3000-\u303F\u3040-\u309F\u30A0-\u30FF",
        r"\u4E00-\u9FFF\uF900-\uFAFF]",                    // single non-alnum non-CJK char
        r")",
    )).unwrap();
}

fn is_cjk(c: char) -> bool {
  matches!(c,
      '\u{3000}'..='\u{303F}' |
      '\u{3040}'..='\u{309F}' |
      '\u{30A0}'..='\u{30FF}' |
      '\u{4E00}'..='\u{9FFF}' |
      '\u{F900}'..='\u{FAFF}'
  )
}

fn classify_token(text: &str) -> TokenClass {
  let mut chars = text.chars();
  let first = match chars.next() {
    Some(c) => c,
    None => return TokenClass::Symbol,
  };
  let is_single = chars.next().is_none();

  if is_single {
    if OPEN_BRACKETS.contains(&first) {
      return TokenClass::BracketOpen;
    }
    if CLOSE_BRACKETS.contains(&first) {
      return TokenClass::BracketClose;
    }
    if DELIMITERS.contains(first) {
      return TokenClass::Delimiter;
    }
  }

  if text.chars().all(|c| c.is_ascii_digit()) {
    return TokenClass::Number;
  }
  if text.chars().all(is_cjk) {
    return TokenClass::Cjk;
  }
  if text.chars().all(|c| c.is_ascii_alphabetic()) {
    return TokenClass::Latin;
  }
  // Ordinals like "1st", "2nd"
  if text.ends_with("st") || text.ends_with("nd") || text.ends_with("rd") || text.ends_with("th") {
    let prefix = &text[..text.len() - 2];
    if !prefix.is_empty() && prefix.chars().all(|c| c.is_ascii_digit()) {
      return TokenClass::Number;
    }
  }

  TokenClass::Symbol
}

fn matching_close(open: char) -> Option<char> {
  BRACKET_PAIRS.iter().find(|(o, _)| *o == open).map(|(_, c)| *c)
}

fn build_bracket_groups(tokens: &mut [Token]) -> Vec<BracketGroup> {
  let mut groups: Vec<BracketGroup> = Vec::new();
  let mut stack: Vec<(usize, char)> = Vec::new(); // (token_index, expected_close)
  let mut depth: u32 = 0;

  for i in 0..tokens.len() {
    match tokens[i].class {
      TokenClass::BracketOpen => {
        depth += 1;
        tokens[i].bracket_depth = depth;
        let ch = tokens[i].content.chars().next().unwrap();
        if let Some(close) = matching_close(ch) {
          stack.push((i, close));
        }
      }
      TokenClass::BracketClose => {
        let ch = tokens[i].content.chars().next().unwrap();
        if let Some(pos) = stack.iter().rposition(|(_, expected)| *expected == ch) {
          let (open_idx, _) = stack.remove(pos);
          let group_index = groups.len();
          let group = BracketGroup {
            group_index,
            open_pos: open_idx,
            close_pos: i,
            is_first: false,
            is_last: false,
            starts_filename: false,
          };
          groups.push(group);

          let open_depth = tokens[open_idx].bracket_depth;
          tokens[open_idx].bracket_group = Some(group_index);
          tokens[i].bracket_group = Some(group_index);
          tokens[i].bracket_depth = open_depth;

          for t in &mut tokens[open_idx + 1..i] {
            if t.bracket_group.is_none() {
              t.bracket_group = Some(group_index);
              t.bracket_depth = open_depth;
            }
          }

          depth = depth.saturating_sub(1);
        } else {
          // Unmatched close bracket
          tokens[i].bracket_depth = depth;
        }
      }
      _ => {
        tokens[i].bracket_depth = depth;
      }
    }
  }

  // Mark first/last and starts_filename
  if !groups.is_empty() {
    groups[0].is_first = true;
    groups.last_mut().unwrap().is_last = true;

    // A bracket group starts the filename if the open bracket is the first token
    // (or only preceded by delimiters)
    if let Some(first) = groups.first_mut() {
      let starts = tokens[..first.open_pos].iter().all(|t| t.class == TokenClass::Delimiter);
      first.starts_filename = starts;
    }
  }

  groups
}

fn build_delimiter_profile(tokens: &[Token], bracket_groups: &[BracketGroup]) -> DelimiterProfile {
  let mut counts: HashMap<char, usize> = HashMap::new();

  for token in tokens {
    if token.class == TokenClass::Delimiter && token.bracket_group.is_none() {
      let ch = token.content.chars().next().unwrap();
      *counts.entry(ch).or_insert(0) += 1;
    }
  }

  let primary = counts.iter().max_by_key(|(_, count)| **count).map(|(&ch, _)| ch);

  let unique_delims = counts.keys().filter(|c| !DASHES.contains(**c)).count();
  let is_uniform = unique_delims <= 1;

  // Detect file index pattern: a number token before the first bracket group or
  // title content
  let file_index_end = detect_file_index(tokens);

  DelimiterProfile {
    primary,
    counts,
    is_uniform,
    file_index_end,
    bracket_groups: bracket_groups.to_vec(),
  }
}

/// Detect a leading file index pattern like "01 - " or "01." before the main
/// content. Returns the end token index (exclusive) of the file index portion.
fn detect_file_index(tokens: &[Token]) -> Option<usize> {
  // Pattern: optional delimiters, then a number, then a delimiter (outside
  // brackets)
  let mut idx = 0;

  // Skip leading delimiters
  while idx < tokens.len() && tokens[idx].class == TokenClass::Delimiter {
    idx += 1;
  }

  if idx >= tokens.len() {
    return None;
  }

  // Expect a number token
  if tokens[idx].class != TokenClass::Number {
    return None;
  }
  let num_idx = idx;
  idx += 1;

  // Check the number is reasonably short (file indices are usually 1-3 digits)
  if tokens[num_idx].content.len() > 3 {
    return None;
  }

  // Expect a delimiter after the number (but not end of tokens)
  if idx >= tokens.len() {
    return None;
  }
  if tokens[idx].class == TokenClass::Delimiter {
    // File index detected: everything up to and including this delimiter
    return Some(idx + 1);
  }

  // Also allow dash-style separators: number + dash + delimiter
  if (tokens[idx].class == TokenClass::Symbol || is_dash_token(&tokens[idx])) && idx + 1 < tokens.len() && tokens[idx + 1].class == TokenClass::Delimiter {
    return Some(idx + 2);
  }

  None
}

fn is_dash_token(token: &Token) -> bool {
  token.content.len() == 1 && DASHES.contains(token.content.chars().next().unwrap())
}

pub fn tokenize(input: &str) -> (Vec<Token>, Vec<BracketGroup>, DelimiterProfile) {
  let mut tokens: Vec<Token> = Vec::new();

  for m in MIXED_TOKEN_REGEX.find_iter(input).filter_map(Result::ok) {
    let text = m.as_str().to_string();
    let span = Span::new(m.start(), m.end());
    let class = classify_token(&text);
    tokens.push(Token::new(span, text, class));
  }

  let bracket_groups = build_bracket_groups(&mut tokens);
  let profile = build_delimiter_profile(&tokens, &bracket_groups);

  (tokens, bracket_groups, profile)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_basic_tokenize() {
    let (tokens, groups, profile) = tokenize("[SubGroup] Anime Name - 01 [720p].mkv");
    assert!(!tokens.is_empty());
    assert!(!groups.is_empty());

    // First token should be an open bracket
    assert_eq!(tokens[0].class, TokenClass::BracketOpen);
    assert_eq!(tokens[0].content, "[");

    // "SubGroup" should be Latin
    assert_eq!(tokens[1].class, TokenClass::Latin);
    assert_eq!(tokens[1].content, "Sub");

    // Find the number token "01"
    let num_token = tokens.iter().find(|t| t.content == "01").unwrap();
    assert_eq!(num_token.class, TokenClass::Number);

    // "mkv" should be Latin
    let ext_token = tokens.iter().find(|t| t.content == "mkv").unwrap();
    assert_eq!(ext_token.class, TokenClass::Latin);

    // Dot before extension should be a delimiter
    let dot_token = tokens.iter().find(|t| t.content == ".").unwrap();
    assert_eq!(dot_token.class, TokenClass::Delimiter);

    // Profile primary delimiter is space
    assert_eq!(profile.primary, Some(' '));
  }

  #[test]
  fn test_cjk_tokenize() {
    let (tokens, groups, _) = tokenize("[字幕组] 進撃の巨人 第01話");
    // CJK bracket
    let cjk_tokens: Vec<_> = tokens.iter().filter(|t| t.class == TokenClass::Cjk).collect();
    assert!(!cjk_tokens.is_empty());

    // Bracket group should exist
    assert!(!groups.is_empty());
  }

  #[test]
  fn test_bracket_groups() {
    let (tokens, groups, _) = tokenize("[Group] Title [Info]");
    assert_eq!(groups.len(), 2);
    assert!(groups[0].is_first);
    assert!(groups[1].is_last);

    // Tokens inside first bracket group should have bracket_group = Some(0)
    for t in &tokens[1..tokens.iter().position(|t| t.content == "]").unwrap()] {
      assert_eq!(t.bracket_group, Some(0));
    }
  }

  #[test]
  fn test_ordinal_number() {
    let (tokens, _, _) = tokenize("2nd Season");
    let ordinal = tokens.iter().find(|t| t.content == "2nd").unwrap();
    assert_eq!(ordinal.class, TokenClass::Number);
  }

  #[test]
  fn test_delimiter_profile_uniform() {
    let (_, _, profile) = tokenize("Some_Anime_Name_01");
    assert!(profile.is_uniform);
    assert_eq!(profile.primary, Some('_'));
  }

  #[test]
  fn test_fullwidth_brackets() {
    let (tokens, groups, _) = tokenize("（Info）Title【Group】");
    assert_eq!(groups.len(), 2);
    let fw_open = tokens.iter().find(|t| t.content == "（").unwrap();
    assert_eq!(fw_open.class, TokenClass::BracketOpen);
  }

  #[test]
  fn test_camel_case_split() {
    let (tokens, _, _) = tokenize("camelCase");
    // Should split into "camel" + "Case"
    let contents: Vec<&str> = tokens.iter().map(|t| t.content.as_str()).collect();
    assert!(contents.contains(&"camel"));
    assert!(contents.contains(&"Case"));
  }

  #[test]
  fn test_file_index_detection() {
    let (_, _, profile) = tokenize("01 - [Group] Anime Title");
    assert!(profile.file_index_end.is_some());
  }

  #[test]
  fn test_empty_input() {
    let (tokens, groups, profile) = tokenize("");
    assert!(tokens.is_empty());
    assert!(groups.is_empty());
    assert!(profile.primary.is_none());
  }
}
