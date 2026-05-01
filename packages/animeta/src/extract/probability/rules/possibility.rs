use crate::extract::probability::{keywords::*, tokenizer::*, types::*};

/// Apply all possibility rules in order.
pub fn apply_all_possibility_rules(tokens: &mut [Token], bracket_groups: &[BracketGroup], profile: &DelimiterProfile, db: &KeywordDb) {
  let input = reconstruct_input(tokens);
  seed_from_keywords(tokens, &input, db);
  assign_bracket_and_delimiter_possibilities(tokens);
  infer_numbers(tokens);
  infer_release_group(tokens, bracket_groups);
  infer_title_candidates(tokens, bracket_groups);
  infer_title_metadata_fragments(tokens);
  assign_zones(tokens, bracket_groups, profile);
}

/// Reconstruct the original input string from token spans for regex matching.
fn reconstruct_input(tokens: &[Token]) -> String {
  if tokens.is_empty() {
    return String::new();
  }
  let max_end = tokens.iter().map(|t| t.span.end).max().unwrap_or(0);
  let mut buf = vec![b' '; max_end];
  for t in tokens {
    for (i, b) in t.content.bytes().enumerate() {
      let pos = t.span.start + i;
      if pos < buf.len() {
        buf[pos] = b;
      }
    }
  }
  String::from_utf8_lossy(&buf).into_owned()
}

/// Look up each token in the keyword database and add possibilities with the
/// corresponding labels at base score 100. Also check regex matches against the
/// full input string.
fn seed_from_keywords(tokens: &mut [Token], input: &str, db: &KeywordDb) {
  for token in tokens.iter_mut() {
    // Exact keyword lookup
    for entry in db.find(&token.content) {
      for &label in entry.labels {
        token.add_possibility(label, 100);
      }
    }

    // Regex matches against the full input anchored to this token's span
    let matches = db.find_regex_matches(input, token.span.start, token.span.end);
    for (label, _start, _end) in matches {
      token.add_possibility(label, 100);
    }
  }
}

/// Add structural possibilities for brackets and delimiters.
fn assign_bracket_and_delimiter_possibilities(tokens: &mut [Token]) {
  for token in tokens.iter_mut() {
    match token.class {
      TokenClass::BracketOpen | TokenClass::BracketClose => {
        token.add_possibility(Label::Bracket, 200);
      }
      TokenClass::Delimiter => {
        token.add_possibility(Label::Delimiter, 200);
        // Single dash characters also get ContextDelimiter possibility
        if token.content.len() == 1 && DASHES.contains(token.content.chars().next().unwrap()) {
          token.add_possibility(Label::ContextDelimiter, 50);
        }
      }
      _ => {}
    }
  }
}

/// Infer numeric possibilities for digit-only tokens.
fn infer_numbers(tokens: &mut [Token]) {
  let last_idx = tokens.len().saturating_sub(1);

  for (i, token) in tokens.iter_mut().enumerate() {
    if token.class != TokenClass::Number {
      continue;
    }
    if token.possibilities.contains_key(&Label::FileChecksum) {
      continue;
    }
    if !token.content.chars().all(|c| c.is_ascii_digit()) {
      continue;
    }

    let content = token.content.clone();

    token.add_possibility(Label::SequenceNumber, 50);

    // 4-digit year range
    if content.len() == 4
      && let Ok(n) = content.parse::<u32>()
      && (1900..=2099).contains(&n)
    {
      token.add_possibility(Label::Year, 50);
    }

    // Common video resolutions
    if let Ok(n) = content.parse::<u32>()
      && matches!(n, 360 | 480 | 576 | 720 | 1080 | 1440 | 2160 | 4320)
    {
      token.add_possibility(Label::VideoResolution, 30);
    }

    // File index at start or end of filename
    if i == 0 || i == last_idx {
      token.add_possibility(Label::FileIndex, 20);
    }
  }
}

/// Infer release group possibilities from bracket groups.
fn infer_release_group(tokens: &mut [Token], bracket_groups: &[BracketGroup]) {
  if bracket_groups.is_empty() {
    return;
  }

  // If the first bracket group starts at position 0 (starts_filename),
  // add ReleaseGroup to all tokens inside it.
  let first = &bracket_groups[0];
  if first.starts_filename {
    for token in tokens[first.open_pos + 1..first.close_pos].iter_mut() {
      token.add_possibility(Label::ReleaseGroup, 150);
    }
  }

  // If the last bracket group is different from the first, and its contents
  // are all unknown (no metadata possibilities), add ReleaseGroup.
  if bracket_groups.len() > 1 {
    let last = bracket_groups.last().unwrap();
    let inner_tokens = &tokens[last.open_pos + 1..last.close_pos];
    let all_unknown = inner_tokens.iter().all(|t| !has_metadata_possibility(t));
    if all_unknown {
      for token in tokens[last.open_pos + 1..last.close_pos].iter_mut() {
        token.add_possibility(Label::ReleaseGroup, 80);
      }
    }
  }
}

/// Check whether a token has any metadata-related possibility (not structural
/// or title/unknown).
fn has_metadata_possibility(token: &Token) -> bool {
  token.possibilities.keys().any(|label| {
    !matches!(
      label,
      Label::Unknown | Label::Title | Label::Bracket | Label::Delimiter | Label::ContextDelimiter | Label::ContextDependent | Label::ReleaseGroup
    )
  })
}

/// Add title candidate possibilities to tokens outside brackets that have no
/// metadata possibilities and are not delimiters.
fn infer_title_candidates(tokens: &mut [Token], bracket_groups: &[BracketGroup]) {
  for token in tokens.iter_mut() {
    if token.bracket_group.is_some() {
      continue;
    }
    if matches!(token.class, TokenClass::Delimiter | TokenClass::BracketOpen | TokenClass::BracketClose) {
      continue;
    }
    if has_metadata_possibility(token) {
      if token.possibilities.contains_key(&Label::SeriesType) {
        token.add_possibility(Label::Title, 100);
      }
      continue;
    }
    token.add_possibility(Label::Title, 50);
  }

  // Also tag title-like bracket groups when an outside-bracket title exists
  // (e.g. "... / 梦想... [年龄限制版] - 09")
  infer_bracket_title(tokens, bracket_groups);
}

fn infer_title_metadata_fragments(tokens: &mut [Token]) {
  let len = tokens.len();
  for i in 0..len {
    if tokens[i].bracket_group.is_some() {
      continue;
    }

    if is_title_bit_depth(tokens, i) {
      tokens[i].add_possibility(Label::Title, 150);
    }

    if is_inline_season_title_fragment(tokens, i) {
      tokens[i].add_possibility(Label::Title, 150);
      if i + 1 < len {
        tokens[i + 1].add_possibility(Label::Title, 150);
      }
    }
  }
}

fn is_title_bit_depth(tokens: &[Token], idx: usize) -> bool {
  let token = &tokens[idx];
  if !token.possibilities.contains_key(&Label::VideoTerm) || !token.content.to_ascii_lowercase().contains("bit") {
    return false;
  }
  find_next_non_delimiter(tokens, idx).is_some_and(|next| tokens[next].possibilities.contains_key(&Label::Title))
}

fn is_inline_season_title_fragment(tokens: &[Token], idx: usize) -> bool {
  if !tokens[idx].content.eq_ignore_ascii_case("s") {
    return false;
  }
  let Some(prev) = find_prev_non_delimiter(tokens, idx) else {
    return false;
  };
  let Some(next) = find_next_non_delimiter(tokens, idx) else {
    return false;
  };
  tokens[prev].possibilities.contains_key(&Label::Title)
    && tokens[next].class == TokenClass::Number
    && tokens[next].content.len() <= 2
}

fn find_prev_non_delimiter(tokens: &[Token], idx: usize) -> Option<usize> {
  (0..idx).rev().find(|&i| tokens[i].class != TokenClass::Delimiter)
}

fn find_next_non_delimiter(tokens: &[Token], idx: usize) -> Option<usize> {
  (idx + 1..tokens.len()).find(|&i| tokens[i].class != TokenClass::Delimiter)
}

/// Detect title inside a bracket group when no outside-bracket title exists.
///
/// Scans bracket groups (skipping the first at position 0 which is typically
/// release group). A group with many unknown tokens and few metadata tokens
/// is a title candidate.
fn infer_bracket_title(tokens: &mut [Token], bracket_groups: &[BracketGroup]) {
  if bracket_groups.len() < 2 {
    return;
  }

  for bg in bracket_groups.iter().skip(if bracket_groups[0].starts_filename { 1 } else { 0 }) {
    let inner = &tokens[bg.open_pos + 1..bg.close_pos];

    let word_tokens: Vec<usize> = inner
      .iter()
      .enumerate()
      .filter(|(_, t)| t.class != TokenClass::Delimiter && t.class != TokenClass::BracketOpen && t.class != TokenClass::BracketClose)
      .map(|(i, _)| i)
      .collect();

    // Single-token groups: e.g. [Movie], [剧场版] — title fragments
    if word_tokens.len() == 1 {
      let idx = word_tokens[0];
      let global = bg.open_pos + 1 + idx;
      let has_prior_media_metadata = has_media_metadata_before(tokens, bg.open_pos);
      let tok = &mut tokens[global];
      if tok.possibilities.contains_key(&Label::SeriesType) {
        tok.add_possibility(Label::Title, 200);
        continue;
      }
      if has_metadata_possibility(tok) {
        continue;
      }
      if has_prior_media_metadata {
        continue;
      }
      tok.add_possibility(Label::Title, 200);
      continue;
    }

    if word_tokens.len() < 2 {
      continue;
    }

    // Count unknown vs metadata tokens
    let unknown_count = word_tokens.iter().filter(|&&i| !has_metadata_possibility(&inner[i])).count();
    let metadata_count = word_tokens.len() - unknown_count;

    let has_theater_edition = inner.iter().any(|t| t.content == "剧场版");

    // If mostly unknown (at least 2 and more than metadata), this is a title
    if unknown_count >= 2 && unknown_count > metadata_count {
      for token in tokens[bg.open_pos + 1..bg.close_pos].iter_mut() {
        token.add_possibility(Label::Title, 200);
      }
    } else if has_theater_edition && unknown_count >= 1 {
      // [剧场版-摇曳露营] — SeriesType + hyphen + title; count 剧场版 as a title fragment
      for token in tokens[bg.open_pos + 1..bg.close_pos].iter_mut() {
        if token.class == TokenClass::Delimiter {
          continue;
        }
        token.add_possibility(Label::Title, 200);
      }
    }
  }
}

fn has_media_metadata_before(tokens: &[Token], end: usize) -> bool {
  tokens[..end].iter().any(|t| {
    t.possibilities.keys().any(|label| {
      matches!(
        label,
        Label::Source
          | Label::VideoResolution
          | Label::VideoTerm
          | Label::AudioTerm
          | Label::SubsTerm
          | Label::Language
          | Label::ReleaseVersion
          | Label::DeviceCompatibility
          | Label::FileChecksum
          | Label::FileExtension
      )
    })
  })
}

/// Assign zone information to each token.
///
/// Zones follow Aniparse's ZoneAssignmentPossibilityRule:
/// - BracketOpen / BracketClose → Zone::Transition
/// - Tokens inside brackets → Zone::Metadata
/// - A context delimiter followed by metadata outside brackets →
///   Zone::Transition for the delimiter, then Zone::Metadata for subsequent
///   tokens
/// - Everything before the first metadata outside brackets → Zone::Title
fn assign_zones(tokens: &mut [Token], bracket_groups: &[BracketGroup], _profile: &DelimiterProfile) {
  // Default: everything starts as Zone::Title (already the default in Token::new)
  // First, mark bracket-related zones
  for bg in bracket_groups {
    if bg.open_pos < tokens.len() {
      tokens[bg.open_pos].zone = Zone::Transition;
    }
    if bg.close_pos < tokens.len() {
      tokens[bg.close_pos].zone = Zone::Transition;
    }
    for token in tokens.get_mut(bg.open_pos + 1..bg.close_pos).unwrap_or(&mut []) {
      token.zone = Zone::Metadata;
    }
  }

  // Walk tokens outside brackets to assign Title vs Metadata zones
  // based on context delimiters
  let mut found_first_metadata = false;
  let mut i = 0;
  while i < tokens.len() {
    // Skip tokens inside brackets (already assigned)
    if tokens[i].bracket_group.is_some() {
      // Bracket groups can signal transition to metadata zone
      if !found_first_metadata {
        found_first_metadata = true;
      }
      i += 1;
      continue;
    }

    if found_first_metadata {
      // After first metadata region, check for context delimiter patterns
      if tokens[i].possibilities.contains_key(&Label::ContextDelimiter) {
        // Look ahead: if followed by metadata-bearing tokens, this is a
        // transition
        let has_following_metadata = tokens[i + 1..].iter().take_while(|t| t.bracket_group.is_none()).any(has_metadata_possibility);
        if has_following_metadata {
          tokens[i].zone = Zone::Transition;
          // Mark subsequent non-bracket tokens as metadata until next
          // delimiter or bracket
          i += 1;
          while i < tokens.len() && tokens[i].bracket_group.is_none() {
            if matches!(tokens[i].class, TokenClass::Delimiter) && !tokens[i].possibilities.contains_key(&Label::ContextDelimiter) {
              break;
            }
            tokens[i].zone = Zone::Metadata;
            i += 1;
          }
          continue;
        }
      }
    }

    // Check if current non-bracket token has metadata possibilities
    if !found_first_metadata && has_metadata_possibility(&tokens[i]) {
      found_first_metadata = true;
      tokens[i].zone = Zone::Metadata;
    }

    i += 1;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::extract::probability::tokenizer::tokenize;

  #[test]
  fn test_seed_from_keywords() {
    let (mut tokens, _groups, _profile) = tokenize("[SubGroup] Title - 01 [HEVC 720p].mkv");
    let input = reconstruct_input(&tokens);
    seed_from_keywords(&mut tokens, &input, &DEFAULT_DB);

    let hevc = tokens.iter().find(|t| t.content == "HEVC").unwrap();
    assert!(hevc.possibilities.contains_key(&Label::VideoTerm));
  }

  #[test]
  fn test_bracket_and_delimiter_possibilities() {
    let (mut tokens, _groups, _profile) = tokenize("[Group] Title");
    assign_bracket_and_delimiter_possibilities(&mut tokens);

    assert!(tokens[0].possibilities.contains_key(&Label::Bracket));
    let space = tokens.iter().find(|t| t.content == " ").unwrap();
    assert!(space.possibilities.contains_key(&Label::Delimiter));
  }

  #[test]
  fn test_infer_numbers_episode() {
    let (mut tokens, _groups, _profile) = tokenize("Title - 01");
    infer_numbers(&mut tokens);

    let num = tokens.iter().find(|t| t.content == "01").unwrap();
    assert!(num.possibilities.contains_key(&Label::SequenceNumber));
  }

  #[test]
  fn test_infer_numbers_year() {
    let (mut tokens, _groups, _profile) = tokenize("Title (2023)");
    infer_numbers(&mut tokens);

    let year = tokens.iter().find(|t| t.content == "2023").unwrap();
    assert!(year.possibilities.contains_key(&Label::Year));
  }

  #[test]
  fn test_infer_release_group_first_bracket() {
    let (mut tokens, groups, _profile) = tokenize("[SubTeam] Title - 01");
    infer_release_group(&mut tokens, &groups);

    let sub = tokens.iter().find(|t| t.content == "Sub").unwrap();
    assert!(sub.possibilities.contains_key(&Label::ReleaseGroup));
  }

  #[test]
  fn test_infer_title_candidates() {
    let (mut tokens, _groups, _profile) = tokenize("[Group] My Anime Title - 01");
    let input = reconstruct_input(&tokens);
    seed_from_keywords(&mut tokens, &input, &DEFAULT_DB);
    assign_bracket_and_delimiter_possibilities(&mut tokens);
    infer_title_candidates(&mut tokens, &_groups);

    let my = tokens.iter().find(|t| t.content == "My").unwrap();
    assert!(my.possibilities.contains_key(&Label::Title));
  }

  #[test]
  fn test_assign_zones_basic() {
    let (mut tokens, groups, profile) = tokenize("[Group] Title [720p]");
    assign_zones(&mut tokens, &groups, &profile);

    // Bracket open should be Transition
    assert_eq!(tokens[0].zone, Zone::Transition);

    // Tokens inside first bracket should be Metadata
    let group_tok = tokens.iter().find(|t| t.content == "Group").unwrap();
    assert_eq!(group_tok.zone, Zone::Metadata);
  }

  #[test]
  fn test_apply_all() {
    let (mut tokens, groups, profile) = tokenize("[SubGroup] Anime Title - 01 [1080p HEVC].mkv");
    apply_all_possibility_rules(&mut tokens, &groups, &profile, &DEFAULT_DB);

    // Verify some basic expectations after all rules
    let hevc = tokens.iter().find(|t| t.content == "HEVC").unwrap();
    assert!(hevc.possibilities.contains_key(&Label::VideoTerm));
    assert_eq!(hevc.zone, Zone::Metadata);

    let num = tokens.iter().find(|t| t.content == "01").unwrap();
    assert!(num.possibilities.contains_key(&Label::SequenceNumber));
  }
}
