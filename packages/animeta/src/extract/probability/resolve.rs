use super::{tokenizer::*, types::*};

pub fn resolve(tokens: &mut [Token]) {
  for token in tokens.iter_mut() {
    if token.possibilities.is_empty() {
      token.resolved = Some(Label::Unknown);
      continue;
    }

    let best = token
      .possibilities
      .iter()
      .max_by(|a, b| a.1.score.cmp(&b.1.score).then_with(|| tiebreak_priority(*a.0).cmp(&tiebreak_priority(*b.0))))
      .map(|(&label, _)| label);

    if let Some(mut winner) = best {
      // Bracket chars always resolve to Bracket unless Title has higher score
      if winner != Label::Bracket && token.possibilities.contains_key(&Label::Bracket) {
        let ch = token.content.chars().next().unwrap_or('\0');
        if OPEN_BRACKETS.contains(&ch) || CLOSE_BRACKETS.contains(&ch) {
          let title_score = token.get_score(Label::Title);
          let rg_score = token.get_score(Label::ReleaseGroup);
          if title_score <= 0 && rg_score <= 0 {
            winner = Label::Bracket;
          }
        }
      }

      // SequenceNumber vs VideoResolution: if VideoResolution has regex-backed
      // higher score, prefer it
      if winner == Label::SequenceNumber
        && let Some(vr) = token.possibilities.get(&Label::VideoResolution)
        && let Some(sn) = token.possibilities.get(&Label::SequenceNumber)
        && vr.score >= sn.score
      {
        winner = Label::VideoResolution;
      }

      // SequenceNumber vs Year: if Year has significantly higher score, prefer it
      if winner == Label::SequenceNumber
        && let Some(year) = token.possibilities.get(&Label::Year)
        && let Some(sn) = token.possibilities.get(&Label::SequenceNumber)
        && year.score > sn.score + 50
      {
        winner = Label::Year;
      }

      token.resolved = Some(winner);
    }
  }
}

fn tiebreak_priority(label: Label) -> u32 {
  match label {
    Label::Bracket => 100,
    Label::Delimiter => 99,
    Label::ContextDelimiter => 98,
    Label::VideoResolution => 90,
    Label::VideoTerm => 89,
    Label::AudioTerm => 88,
    Label::Source => 87,
    Label::Language => 86,
    Label::SubsTerm => 85,
    Label::FileChecksum => 84,
    Label::ReleaseInformation => 83,
    Label::ReleaseVersion => 82,
    Label::DeviceCompatibility => 81,
    Label::EpisodePrefix => 70,
    Label::SeasonPrefix => 69,
    Label::VolumePrefix => 68,
    Label::ContentType => 67,
    Label::SeriesType => 66,
    Label::SequenceNumber => 60,
    Label::SequenceRange => 59,
    Label::Year => 55,
    Label::ReleaseGroup => 50,
    Label::Title => 40,
    Label::FileIndex => 30,
    Label::ContentIdentifier => 25,
    Label::ContextDependent => 20,
    Label::FileExtension => 15,
    Label::Other => 10,
    _ => 0,
  }
}

pub fn calculate_confidence(tokens: &[Token]) -> f64 {
  let mut margins = Vec::new();
  for token in tokens {
    if token.possibilities.is_empty() {
      continue;
    }
    let mut scores: Vec<i32> = token.possibilities.values().map(|p| p.score).collect();
    scores.sort_unstable_by(|a, b| b.cmp(a));
    let winner = scores[0] as f64;
    let runner_up = scores.get(1).copied().unwrap_or(0) as f64;
    margins.push(winner - runner_up);
  }
  if margins.is_empty() {
    return 0.0;
  }
  margins.iter().sum::<f64>() / margins.len() as f64
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::extract::probability::{
    keywords::DEFAULT_DB,
    rules::{possibility::apply_all_possibility_rules, score::apply_all_score_rules},
    tokenizer::tokenize,
  };

  #[test]
  fn test_resolve_basic() {
    let (mut tokens, groups, profile) = tokenize("[Group] Title - 01 [720p]");
    apply_all_possibility_rules(&mut tokens, &groups, &profile, &DEFAULT_DB);
    apply_all_score_rules(&mut tokens, &groups);
    resolve(&mut tokens);

    let bracket = tokens.iter().find(|t| t.content == "[").unwrap();
    assert_eq!(bracket.resolved, Some(Label::Bracket));

    let num = tokens.iter().find(|t| t.content == "01").unwrap();
    assert_eq!(num.resolved, Some(Label::SequenceNumber));
  }

  #[test]
  fn test_confidence_positive() {
    let (mut tokens, groups, profile) = tokenize("[Group] Title - 01 [720p]");
    apply_all_possibility_rules(&mut tokens, &groups, &profile, &DEFAULT_DB);
    apply_all_score_rules(&mut tokens, &groups);
    resolve(&mut tokens);

    let confidence = calculate_confidence(&tokens);
    assert!(confidence > 0.0);
  }
}
