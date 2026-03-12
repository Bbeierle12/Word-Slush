const PUNCTUATION: &[char] = &['.', ',', '!', '?', ';', ':', '"', '\'', '(', ')', '[', ']', '{', '}'];

pub fn tokenize(text: &str, normalize: bool) -> Vec<String> {
    text.split_whitespace()
        .map(|word| {
            let stripped: String = word.chars().filter(|c| !PUNCTUATION.contains(c)).collect();
            if normalize {
                stripped.to_lowercase()
            } else {
                stripped
            }
        })
        .filter(|w| !w.is_empty())
        .collect()
}
