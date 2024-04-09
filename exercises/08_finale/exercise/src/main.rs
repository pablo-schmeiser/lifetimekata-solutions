use std::option;

use require_lifetimes::require_lifetimes;

#[derive(Debug, PartialEq, Eq)]
enum MatcherToken <'a> {
    /// This is just text without anything special.
    RawText(&'a str),
    /// This is when text could be any one of multiple
    /// strings. It looks like `(one|two|three)`, where
    /// `one`, `two` or `three` are the allowed strings.
    OneOfText(Vec<&'a str>),
    /// This is when you're happy to accept any single character.
    /// It looks like `.`
    WildCard,
}

#[derive(Debug, PartialEq, Eq)]
struct Matcher <'a> {
    /// This is the actual text of the matcher
    text: &'a str,
    /// This is a vector of the tokens inside the expression.
    tokens: Vec<MatcherToken<'a>>,
    /// This keeps track of the most tokens that this matcher has matched.
    most_tokens_matched: usize,
}

impl<'a> Matcher<'a> {
    /// This should take a string reference, and return
    /// an `Matcher` which has parsed that reference.
    #[require_lifetimes]
    fn new(text: &'a str) -> Option<Matcher<'a>> {
        let mut tokens: Vec<MatcherToken> = vec![];
        let mut unmatched = text;

        loop {
            if unmatched.is_empty() {
                break;
            } else if unmatched.starts_with('.') {
                tokens.push(MatcherToken::WildCard);
                unmatched = &unmatched[1..];
            } else if unmatched.starts_with('(') {
                let fc = unmatched.find(')')?;
                let (options, leftover) = unmatched.split_at(fc);
                tokens.push(MatcherToken::OneOfText(options[1..].split('|').collect()));
                unmatched = &leftover[1..];
            } else {
                let first_wc = unmatched.find('.').unwrap_or(unmatched.len());
                let first_one_of = unmatched.find('(').unwrap_or(unmatched.len());
                let first_token = first_wc.min(first_one_of);
                tokens.push(MatcherToken::RawText(&unmatched[..first_token]));
                unmatched = &unmatched[first_token..];
            }
        }

        eprintln!("{tokens:?}");

        Some(Matcher { text, tokens, most_tokens_matched: 0 })
    }

    /// This should take a string, and return a vector of tokens, and the corresponding part
    /// of the given string. For examples, see the test cases below.
    #[require_lifetimes]
    fn match_string <'b, 'c> (&'b mut self, string: &'c str) -> Vec<(&'b MatcherToken<'a>, &'c str)> {
        let mut unmatched = string;
        let mut answer = vec![];

        'outer_loop: for token in self.tokens.iter() {
            if unmatched.is_empty() {
                break;
            }
            
            match token {
                MatcherToken::WildCard => {
                    let offset = unmatched.chars().next().unwrap().len_utf8();
                    answer.push((token, &unmatched[..offset]));
                    unmatched = &unmatched[offset..];
                }

                MatcherToken::OneOfText(options) => {
                    for start in options {
                        if unmatched.starts_with(start) {
                            let split = unmatched.split_at(start.len());
                            answer.push((token, split.0));
                            unmatched = split.1;
                            continue 'outer_loop;
                        }
                    }
                    break;
                }

                MatcherToken::RawText(text) => {
                    if unmatched.starts_with(text) {
                        let split = unmatched.split_at(text.len());
                        answer.push((token, split.0));
                        unmatched = split.1;
                        continue 'outer_loop;
                    } else {
                        break;
                    }
                }
            }
        }

        if answer.len() > self.most_tokens_matched {
            self.most_tokens_matched = answer.len();
        }

        answer
    }
}

fn main() {
    unimplemented!()
}

#[cfg(test)]
mod test {
    use super::{Matcher, MatcherToken};
    #[test]
    fn simple_test() {
        let match_string = "abc(d|e|f).".to_string();
        let mut matcher = Matcher::new(&match_string).unwrap();

        assert_eq!(matcher.most_tokens_matched, 0);

        {
            let candidate1 = "abcge".to_string();
            let result = matcher.match_string(&candidate1);
            assert_eq!(result, vec![(&MatcherToken::RawText("abc"), "abc"),]);
            assert_eq!(matcher.most_tokens_matched, 1);
        }

        {
            // Change 'e' to '💪' if you want to test unicode.
            let candidate1 = "abcd💪".to_string();
            let result = matcher.match_string(&candidate1);
            assert_eq!(
                result,
                vec![
                    (&MatcherToken::RawText("abc"), "abc"),
                    (&MatcherToken::OneOfText(vec!["d", "e", "f"]), "d"),
                    (&MatcherToken::WildCard, "💪") // or '💪'
                ]
            );
            assert_eq!(matcher.most_tokens_matched, 3);
        }
    }

    #[test]
    fn broken_matcher() {
        let match_string = "abc(d|e|f.".to_string();
        let matcher = Matcher::new(&match_string);
        assert_eq!(matcher, None);
    }
}
