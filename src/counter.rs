use serde::Serialize;
use std::collections::HashMap;

const STOP_WORDS: &[&str] = &[
    // Articles / determiners
    "a", "an", "the", "this", "that", "these", "those",
    "some", "any", "all", "each", "every", "no", "other",
    // Pronouns
    "i", "me", "my", "mine", "myself",
    "you", "your", "yours", "yourself",
    "he", "him", "his", "himself",
    "she", "her", "hers", "herself",
    "it", "its", "itself",
    "we", "us", "our", "ours", "ourselves",
    "they", "them", "their", "theirs", "themselves",
    "what", "which", "who", "whom", "whose",
    // Be / have / do
    "am", "is", "are", "was", "were", "be", "been", "being",
    "have", "has", "had", "having",
    "do", "does", "did", "doing",
    // Modals
    "can", "could", "will", "would", "shall", "should",
    "may", "might", "must",
    // Prepositions
    "in", "on", "at", "to", "of", "for", "with", "by",
    "from", "into", "through", "during", "before", "after",
    "above", "below", "between", "under", "over", "out",
    "about", "up", "down", "off", "as", "via",
    // Conjunctions
    "and", "or", "but", "nor", "yet", "so", "because",
    "if", "unless", "until", "while", "when", "where",
    "than", "then",
    // Common adverbs
    "not", "no", "yes", "very", "just", "also", "only",
    "even", "too", "much", "more", "most", "less", "least",
    "here", "there", "now", "how", "why",
    // Contractions (after apostrophe stripping)
    "dont", "doesnt", "didnt", "isnt", "arent", "wasnt", "werent",
    "wont", "wouldnt", "cant", "couldnt", "shouldnt",
    "havent", "hasnt", "hadnt", "im", "ive", "ill",
    "youre", "youve", "youll", "hes", "shes", "its",
    "were", "weve", "theyre", "theyve", "theyll",
    "thats", "whats", "lets",
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
