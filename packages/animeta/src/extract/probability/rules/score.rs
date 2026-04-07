use crate::extract::probability::types::*;

pub fn apply_all_score_rules(tokens: &mut [Token], bracket_groups: &[BracketGroup]) {
  score_release_group(tokens, bracket_groups);
  score_sequence_numbers(tokens);
  score_metadata_in_brackets(tokens);
  score_title_tokens(tokens, bracket_groups);
}

fn score_release_group(tokens: &mut [Token], bracket_groups: &[BracketGroup]) {
  if bracket_groups.is_empty() {
    return;
  }
  let first = &bracket_groups[0];
  if first.starts_filename {
    for token in tokens[first.open_pos + 1..first.close_pos].iter_mut() {
      token.add_score(Label::ReleaseGroup, 100, "first_bracket_at_pos0");
      if token.possibilities.contains_key(&Label::SequenceNumber) {
        token.add_score(Label::SequenceNumber, -100, "seq_unlikely_in_first_bracket");
      }
      if token.possibilities.contains_key(&Label::Year) {
        token.add_score(Label::Year, -50, "year_unlikely_in_first_bracket");
      }
    }
  }
}

fn score_sequence_numbers(tokens: &mut [Token]) {
  let len = tokens.len();
  for i in 0..len {
    if !tokens[i].possibilities.contains_key(&Label::SequenceNumber) {
      continue;
    }
    if !tokens[i].content.chars().all(|c| c.is_ascii_digit()) {
      continue;
    }

    // Check if any delimiter/dash between this number and the previous
    // meaningful token is a context delimiter (dash)
    let has_preceding_dash = (0..i)
      .rev()
      .take_while(|&j| tokens[j].class == TokenClass::Delimiter)
      .any(|j| tokens[j].possibilities.contains_key(&Label::ContextDelimiter));

    if has_preceding_dash {
      tokens[i].add_score(Label::SequenceNumber, 50, "after_context_delimiter");
    }

    // Boost if preceded by episode/season/volume prefix
    if let Some(prev) = find_prev_non_delimiter(tokens, i) {
      if tokens[prev].possibilities.contains_key(&Label::EpisodePrefix) {
        tokens[i].add_score(Label::SequenceNumber, 100, "after_episode_prefix");
      }
      if tokens[prev].possibilities.contains_key(&Label::SeasonPrefix) {
        tokens[i].add_score(Label::SequenceNumber, 100, "after_season_prefix");
      }
      if tokens[prev].possibilities.contains_key(&Label::VolumePrefix) {
        tokens[i].add_score(Label::SequenceNumber, 80, "after_volume_prefix");
      }
    }

    // Boost if followed by context delimiter or bracket
    if let Some(next) = find_next_non_delimiter(tokens, i) {
      if tokens[next].possibilities.contains_key(&Label::Bracket) {
        tokens[i].add_score(Label::SequenceNumber, 25, "before_bracket");
      }
      if tokens[next].possibilities.contains_key(&Label::ContextDelimiter) {
        tokens[i].add_score(Label::SequenceNumber, 25, "before_context_delimiter");
      }
    }

    // Boost if inside brackets alone (common [01] pattern)
    if tokens[i].bracket_group.is_some() {
      let in_bracket_alone = is_alone_in_bracket(tokens, i);
      if in_bracket_alone {
        tokens[i].add_score(Label::SequenceNumber, 50, "alone_in_bracket");
      }
    }

    // Penalize large numbers as episodes (>1000 unlikely)
    if let Ok(n) = tokens[i].content.parse::<u32>()
      && n > 1000
    {
      tokens[i].add_score(Label::SequenceNumber, -50, "large_number_unlikely_episode");
    }

    // Not an episode index: month (4月), anniversary (15周年), bit depth (16bit)
    if let Some(next) = find_next_non_delimiter(tokens, i) {
      if tokens[next].content == "月" {
        tokens[i].add_score(Label::SequenceNumber, -200, "month_prefix_not_episode");
      }
      if tokens[next].content == "周年" {
        tokens[i].add_score(Label::SequenceNumber, -200, "anniversary_not_episode");
      }
      if tokens[next].content.eq_ignore_ascii_case("bit") {
        tokens[i].add_score(Label::SequenceNumber, -200, "bit_depth_not_episode");
      }
    }
  }
}

fn score_metadata_in_brackets(tokens: &mut [Token]) {
  for i in 0..tokens.len() {
    if tokens[i].bracket_group.is_none() {
      continue;
    }
    let metadata_labels = [
      Label::VideoTerm,
      Label::AudioTerm,
      Label::VideoResolution,
      Label::Source,
      Label::Language,
      Label::SubsTerm,
      Label::ReleaseInformation,
      Label::FileChecksum,
      Label::DeviceCompatibility,
    ];
    for &label in &metadata_labels {
      if tokens[i].possibilities.contains_key(&label) {
        tokens[i].add_score(label, 25, "metadata_in_bracket");
      }
    }

    // Check neighbor boost: adjacent metadata tokens reinforce each other
    if has_any_metadata(&tokens[i]) {
      let has_prev_meta = i > 0 && has_any_metadata(&tokens[i - 1]);
      let has_next_meta = i + 1 < tokens.len() && has_any_metadata(&tokens[i + 1]);
      if has_prev_meta || has_next_meta {
        for &label in &metadata_labels {
          if tokens[i].possibilities.contains_key(&label) {
            tokens[i].add_score(label, 25, "neighbor_metadata_boost");
          }
        }
      }
    }
  }
}

fn score_title_tokens(tokens: &mut [Token], bracket_groups: &[BracketGroup]) {
  let first_content_start = bracket_groups.first().filter(|bg| bg.starts_filename).map(|bg| bg.close_pos + 1).unwrap_or(0);

  for i in 0..tokens.len() {
    if !tokens[i].possibilities.contains_key(&Label::Title) {
      continue;
    }

    // Title tokens right after the first bracket group get a big boost
    if i >= first_content_start && tokens[i].zone == Zone::Title {
      tokens[i].add_score(Label::Title, 50, "in_title_zone");
    }

    // Consecutive unknown tokens (no metadata) outside brackets form title
    let is_isolated = {
      let prev_is_title = i > 0 && tokens[i - 1].possibilities.contains_key(&Label::Title);
      let next_is_title = i + 1 < tokens.len() && tokens[i + 1].possibilities.contains_key(&Label::Title);
      let prev_is_delim = i > 0 && tokens[i - 1].class == TokenClass::Delimiter;
      let next_is_delim = i + 1 < tokens.len() && tokens[i + 1].class == TokenClass::Delimiter;
      prev_is_title || next_is_title || prev_is_delim || next_is_delim
    };
    if is_isolated {
      tokens[i].add_score(Label::Title, 25, "adjacent_to_title_or_delim");
    }
  }
}

fn find_prev_non_delimiter(tokens: &[Token], idx: usize) -> Option<usize> {
  (0..idx).rev().find(|&i| tokens[i].class != TokenClass::Delimiter)
}

fn find_next_non_delimiter(tokens: &[Token], idx: usize) -> Option<usize> {
  (idx + 1..tokens.len()).find(|&i| tokens[i].class != TokenClass::Delimiter)
}

fn is_alone_in_bracket(tokens: &[Token], idx: usize) -> bool {
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

fn has_any_metadata(token: &Token) -> bool {
  token.possibilities.keys().any(|label| {
    matches!(
      label,
      Label::VideoTerm
        | Label::AudioTerm
        | Label::VideoResolution
        | Label::Source
        | Label::Language
        | Label::SubsTerm
        | Label::ReleaseInformation
        | Label::FileChecksum
        | Label::DeviceCompatibility
    )
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::extract::probability::{keywords::DEFAULT_DB, rules::possibility::apply_all_possibility_rules, tokenizer::tokenize};

  #[test]
  fn test_release_group_score() {
    let (mut tokens, groups, profile) = tokenize("[SubGroup] Title - 01");
    apply_all_possibility_rules(&mut tokens, &groups, &profile, &DEFAULT_DB);
    apply_all_score_rules(&mut tokens, &groups);

    let sub = tokens.iter().find(|t| t.content == "Sub").unwrap();
    assert!(sub.get_score(Label::ReleaseGroup) > 200);
  }

  #[test]
  fn test_sequence_number_after_dash() {
    let (mut tokens, groups, profile) = tokenize("[Group] Title - 01");
    apply_all_possibility_rules(&mut tokens, &groups, &profile, &DEFAULT_DB);
    apply_all_score_rules(&mut tokens, &groups);

    let num = tokens.iter().find(|t| t.content == "01").unwrap();
    assert!(num.get_score(Label::SequenceNumber) > 50);
  }

  #[test]
  fn test_metadata_in_bracket_boost() {
    let (mut tokens, groups, profile) = tokenize("[Group] Title [1080p HEVC]");
    apply_all_possibility_rules(&mut tokens, &groups, &profile, &DEFAULT_DB);
    apply_all_score_rules(&mut tokens, &groups);

    let hevc = tokens.iter().find(|t| t.content == "HEVC").unwrap();
    assert!(hevc.get_score(Label::VideoTerm) > 100);
  }
}
