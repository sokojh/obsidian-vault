use tantivy::tokenizer::{BoxTokenStream, LowerCaser, TextAnalyzer, Token, TokenStream, Tokenizer};

const TOKENIZER_NAME: &str = "ov_cjk";

/// CJK-aware tokenizer: uses bigrams for CJK characters, whitespace split for Latin
#[derive(Clone)]
pub struct CjkBigramTokenizer;

impl Tokenizer for CjkBigramTokenizer {
    type TokenStream<'a> = BoxTokenStream<'a>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        let mut tokens = Vec::new();
        let mut latin_start: Option<usize> = None;

        for (i, ch) in text.char_indices() {
            if is_cjk(ch) {
                // Flush any pending Latin token
                if let Some(start) = latin_start.take() {
                    let word = &text[start..i];
                    let trimmed = word.trim();
                    if !trimmed.is_empty() {
                        tokens.push(Token {
                            offset_from: start,
                            offset_to: i,
                            position: tokens.len(),
                            text: trimmed.to_lowercase(),
                            position_length: 1,
                        });
                    }
                }

                // Single CJK character as token (unigram)
                tokens.push(Token {
                    offset_from: i,
                    offset_to: i + ch.len_utf8(),
                    position: tokens.len(),
                    text: ch.to_string(),
                    position_length: 1,
                });
            } else if ch.is_whitespace() || ch.is_ascii_punctuation() {
                // Flush Latin token
                if let Some(start) = latin_start.take() {
                    let word = &text[start..i];
                    let trimmed = word.trim();
                    if !trimmed.is_empty() {
                        tokens.push(Token {
                            offset_from: start,
                            offset_to: i,
                            position: tokens.len(),
                            text: trimmed.to_lowercase(),
                            position_length: 1,
                        });
                    }
                }
            } else if latin_start.is_none() {
                latin_start = Some(i);
            }
        }

        // Flush remaining Latin token
        if let Some(start) = latin_start {
            let word = &text[start..];
            let trimmed = word.trim();
            if !trimmed.is_empty() {
                tokens.push(Token {
                    offset_from: start,
                    offset_to: text.len(),
                    position: tokens.len(),
                    text: trimmed.to_lowercase(),
                    position_length: 1,
                });
            }
        }

        BoxTokenStream::new(VecTokenStream { tokens, index: 0 })
    }
}

struct VecTokenStream {
    tokens: Vec<Token>,
    index: usize,
}

impl TokenStream for VecTokenStream {
    fn advance(&mut self) -> bool {
        if self.index < self.tokens.len() {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn token(&self) -> &Token {
        &self.tokens[self.index - 1]
    }

    fn token_mut(&mut self) -> &mut Token {
        &mut self.tokens[self.index - 1]
    }
}

fn is_cjk(c: char) -> bool {
    matches!(c,
        '\u{4E00}'..='\u{9FFF}' |   // CJK Unified Ideographs
        '\u{3400}'..='\u{4DBF}' |   // CJK Extension A
        '\u{AC00}'..='\u{D7AF}' |   // Hangul Syllables
        '\u{1100}'..='\u{11FF}' |   // Hangul Jamo
        '\u{3130}'..='\u{318F}' |   // Hangul Compatibility Jamo
        '\u{3000}'..='\u{303F}' |   // CJK Symbols
        '\u{30A0}'..='\u{30FF}' |   // Katakana
        '\u{3040}'..='\u{309F}'     // Hiragana
    )
}

/// Build a text analyzer with CJK bigram support
pub fn build_text_analyzer() -> TextAnalyzer {
    TextAnalyzer::builder(CjkBigramTokenizer)
        .filter(LowerCaser)
        .build()
}

/// Get the tokenizer name used in schema
pub fn tokenizer_name() -> &'static str {
    TOKENIZER_NAME
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cjk_tokenizer_korean() {
        let mut tok = CjkBigramTokenizer;
        let mut stream = tok.token_stream("쿠버네티스 기초");
        let mut tokens = Vec::new();
        while stream.advance() {
            tokens.push(stream.token().text.clone());
        }
        // Each Korean character should be a separate token
        assert!(tokens.contains(&"쿠".to_string()));
        assert!(tokens.contains(&"버".to_string()));
        assert!(tokens.contains(&"기".to_string()));
        assert!(tokens.contains(&"초".to_string()));
    }

    #[test]
    fn test_cjk_tokenizer_mixed() {
        let mut tok = CjkBigramTokenizer;
        let mut stream = tok.token_stream("Kubernetes 기초 guide");
        let mut tokens = Vec::new();
        while stream.advance() {
            tokens.push(stream.token().text.clone());
        }
        assert!(tokens.contains(&"kubernetes".to_string()));
        assert!(tokens.contains(&"guide".to_string()));
        assert!(tokens.contains(&"기".to_string()));
        assert!(tokens.contains(&"초".to_string()));
    }
}
