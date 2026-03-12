use serde::Serialize;
use std::collections::HashMap;

const STOP_WORDS: &[&str] = &[
    "the", "a", "an", "is", "are", "was", "were", "i", "you", "it",
    "in", "on", "at", "to", "of", "and", "or", "but", "for", "with",
    "that", "this", "my", "your",
];

#[derive(Serialize)]
pub struct WordCount {
    pub word: String,
    pub count: u32,
}

pub fn count_words(tokens: Vec<String>, filter_stop_words: bool) -> Vec<WordCount> {
    let mut map: HashMap<String, u32> = HashMap::new();

    for token in tokens {
        if filter_stop_words && STOP_WORDS.contains(&token.to_lowercase().as_str()) {
            continue;
        }
        *map.entry(token).or_insert(0) += 1;
    }

    let mut counts: Vec<WordCount> = map
        .into_iter()
        .map(|(word, count)| WordCount { word, count })
        .collect();

    counts.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.word.cmp(&b.word)));
    counts
}
