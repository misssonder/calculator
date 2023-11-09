use crate::error::{Error, Result};
use std::fmt::{Display, Formatter};
use std::iter::Peekable;
use std::str::Chars;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Token {
    Number(String),
    Asterisk,
    Caret,
    CloseParen,
    Equal,
    Exclamation,
    GreaterThan,
    GreaterThanOrEqual,
    LessOrGreaterThan,
    LessThan,
    LessThanOrEqual,
    Minus,
    OpenParen,
    Percent,
    Plus,
    Slash,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Token::Number(s) => s,
            Token::Asterisk => "*",
            Token::Caret => "^",
            Token::Equal => "=",
            Token::GreaterThan => ">",
            Token::GreaterThanOrEqual => ">=",
            Token::LessOrGreaterThan => "<>",
            Token::LessThan => "<",
            Token::LessThanOrEqual => "<=",
            Token::Minus => "-",
            Token::Percent => "%",
            Token::Plus => "+",
            Token::Slash => "/",
            Token::OpenParen => "(",
            Token::CloseParen => ")",
            Token::Exclamation => "!",
        })
    }
}

pub(crate) struct Lexer<'a> {
    iter: Peekable<Chars<'a>>,
}

impl Iterator for Lexer<'_> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        self.consume_space();
        match self.scan() {
            Some(token) => Some(Ok(token)),
            None => self
                .iter
                .next()
                .map(|c| Err(Error::Parse(format!("Unexpected character {}", c)))),
        }
    }
}

impl Lexer<'_> {
    pub(crate) fn new(input: &str) -> Lexer {
        Lexer {
            iter: input.chars().peekable(),
        }
    }

    fn next_if<F: Fn(char) -> bool>(&mut self, predicate: F) -> Option<char> {
        self.iter.peek().filter(|&c| predicate(*c))?;
        self.iter.next()
    }

    fn next_while<F: Fn(char) -> bool>(&mut self, predicate: F) -> Option<String> {
        let mut value = String::new();
        while let Some(c) = self.next_if(&predicate) {
            value.push(c)
        }
        Some(value)
    }

    fn consume_space(&mut self) {
        self.next_while(|c| c.is_whitespace());
    }

    fn next_if_token<F: Fn(char) -> Option<Token>>(&mut self, tokenizer: F) -> Option<Token> {
        let token = self.iter.peek().and_then(|c| tokenizer(*c))?;
        self.iter.next();
        Some(token)
    }

    fn scan(&mut self) -> Option<Token> {
        self.consume_space();
        match self.iter.peek() {
            Some(c) if c.is_ascii_digit() => self.scan_number(),
            Some(_) => self.scan_symbol(),
            None => None,
        }
    }

    fn scan_number(&mut self) -> Option<Token> {
        let mut num = self.next_while(|c| c.is_ascii_digit())?;
        if let Some(sep) = self.next_if(|c| c == '.') {
            num.push(sep)
        }
        if let Some(dec) = self.next_while(|c| c.is_ascii_digit()) {
            num.push_str(&dec);
        }
        Some(Token::Number(num))
    }

    fn scan_symbol(&mut self) -> Option<Token> {
        self.next_if_token(|c| match c {
            '=' => Some(Token::Equal),
            '>' => Some(Token::GreaterThan),
            '<' => Some(Token::LessThan),
            '+' => Some(Token::Plus),
            '-' => Some(Token::Minus),
            '*' => Some(Token::Asterisk),
            '/' => Some(Token::Slash),
            '^' => Some(Token::Caret),
            '%' => Some(Token::Percent),
            '(' => Some(Token::OpenParen),
            ')' => Some(Token::CloseParen),
            '!' => Some(Token::Exclamation),
            _ => None,
        })
        .map(|token| match token {
            Token::LessThan => {
                if self.next_if(|c| c == '>').is_some() {
                    Token::LessOrGreaterThan
                } else if self.next_if(|c| c == '=').is_some() {
                    Token::LessThanOrEqual
                } else {
                    token
                }
            }
            Token::GreaterThan => {
                if self.next_if(|c| c == '=').is_some() {
                    Token::GreaterThanOrEqual
                } else {
                    token
                }
            }
            _ => token,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        {
            let lexer = Lexer::new("1 + 1");
            for token in lexer {
                assert!(token.is_ok())
            }
        }
        {
            let mut lexer = Lexer::new("1 + +m+");
            assert!(lexer.next().unwrap().is_ok());
            assert!(lexer.next().unwrap().is_ok());
            assert!(lexer.next().unwrap().is_ok());
            assert!(lexer.next().unwrap().is_err());
            assert!(lexer.next().unwrap().is_ok());
        }
        {
            let lexer = Lexer::new("1.2++=+");
            let left: Vec<_> = lexer.collect();
            assert_eq!(
                left,
                vec![
                    Ok(Token::Number("1.2".into())),
                    Ok(Token::Plus),
                    Ok(Token::Plus),
                    Ok(Token::Equal),
                    Ok(Token::Plus),
                ]
            );
        }
    }
}
