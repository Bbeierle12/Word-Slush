/// Characters to strip from tokens entirely.
const STRIP_CHARS: &[char] = &[
    // Standard punctuation
    '.', ',', '!', '?', ';', ':', '"', '\'',
    // Brackets
    '(', ')', '[', ']', '{', '}', '<', '>',
    // Markdown / code
    '`', '*', '_', '#', '~',
    // Quotes (smart/curly)
    '\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}', // ' ' " "
    '\u{00AB}', '\u{00BB}', // « »
    // Misc
    '\\', '/', '|', '@', '^', '+', '=',
];

/// Characters that should split a token into multiple words (not just strip).
const SPLIT_CHARS: &[char] = &[
    '-',          // hyphen
    '\u{2013}',   // en-dash –
    '\u{2014}',   // em-dash —
];

pub fn tokenize(text: &str, normalize: bool) -> Vec<String> {
    let mut tokens = Vec::new();

    for word in text.split_whitespace() {
        // First, split on dash/em-dash boundaries to separate hyphenated words
        let sub_tokens = split_on_chars(word, SPLIT_CHARS);

        for sub in sub_tokens {
            let stripped: String = sub.chars().filter(|c: &char| !STRIP_CHARS.contains(c)).collect();
            if stripped.is_empty() {
                continue;
            }
            if normalize {
                tokens.push(stripped.to_lowercase());
            } else {
                tokens.push(stripped);
            }
        }
    }

    tokens
}

/// Split a string on any of the given characters, returning non-empty fragments.
fn split_on_chars<'a>(s: &'a str, chars: &[char]) -> Vec<&'a str> {
    s.split(|c: char| chars.contains(&c))
        .filter(|part| !part.is_empty())
        .collect()
}
