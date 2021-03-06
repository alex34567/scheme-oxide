/*
    Copyright 2018-2019 Alexander Eckhart

    This file is part of scheme-oxide.

    Scheme-oxide is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Scheme-oxide is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with scheme-oxide.  If not, see <https://www.gnu.org/licenses/>.
*/

use regex::Regex;

use lazy_static::lazy_static;

#[derive(Debug, Eq, PartialEq)]
pub enum Block {
    Start,
    End,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Mark {
    Quote,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Token<'a> {
    Block(Block),
    TString(&'a str),
    Symbol(&'a str),
    Number(&'a str),
    Bool(bool),
    Dot,
    Mark(Mark),
}

fn gen_regex() -> Regex {
    let comment = "(?:;.*)";
    let whitespace = format!("(?:[[:space:]]|{})", comment);

    let delmer = format!(r#"(?:{}|[()";]|$)"#, whitespace);

    let special_initial = "[!$%&*/:<=>?^_~]";
    let special_subsequent = "[+.@-]";
    let initial = format!("(?:[[:alpha:]]|{})", special_initial);
    let subsequent = format!("(?:[0-9]|{}|{})", initial, special_subsequent);
    let normal_symbol = format!("(?:{}{}*)", initial, subsequent);

    let odd_symbol = r"(?:[+-]|\.{3})";
    let symbol = format!("(?:(?P<symbol>{}|{}){})", normal_symbol, odd_symbol, delmer);

    let string_body = |id| format!(r#"(?P<{}Body>(?:[^"\\\n]|\\.)*)"#, id);
    let good_string = format!(r#"(?:"{}")"#, string_body("goodString"));
    let bad_eof_string = format!(r#"(?:"{}\\?$)"#, string_body("badEofString"));

    let number = format!(r"(?:(?P<number>(?:\+|-)?[0-9]+){})", delmer);

    let block = r"(?P<block>\(|\))";

    let boolean = format!("(?:(?P<boolean>#t|#f){})", delmer);

    let dot = format!(r"(?:(?P<dot>\.){})", delmer);

    let mark = "(?P<mark>')";

    //Matches any multi character sequence cut off by end of buffer
    let clipped = r"(?P<clipped>(?:\.{2}|#)$)";

    let regex_str = format!(
        "^(?:{}|{}|{}|{}|(?P<whitespace>{}+)|{}|{}|{}|{}|{})",
        number, symbol, good_string, block, whitespace, bad_eof_string, clipped, boolean, dot, mark
    );

    Regex::new(&regex_str).unwrap()
}

lazy_static! {
    static ref REGEX: Regex = gen_regex();
}

//Type used to store more information about each token than is exposed to parser
enum InternalToken<'a> {
    PublicToken(Token<'a>),
    EndOfFile,
    Whitespace,
}

impl<'a> InternalToken<'a> {
    fn can_ignore(&self) -> bool {
        match self {
            InternalToken::PublicToken(_) => false,
            InternalToken::EndOfFile => false,
            InternalToken::Whitespace => true,
        }
    }

    fn into_option(self) -> Option<Token<'a>> {
        match self {
            InternalToken::PublicToken(token) => Some(token),
            _ => None,
        }
    }

    fn into_public(self) -> Token<'a> {
        self.into_option().unwrap()
    }
}

#[derive(Debug)]
pub enum TokenizerError {
    UnexpectedEndOfFile,
    UnknownToken,
}

pub struct Tokenizer<'a> {
    input: &'a str,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Tokenizer { input }
    }

    fn gen_token(&mut self) -> Result<InternalToken<'a>, TokenizerError> {
        if self.input.is_empty() {
            return Ok(InternalToken::EndOfFile);
        }

        let unchecked_captures = REGEX.captures(&self.input);
        let captures = if let Some(cap) = unchecked_captures {
            cap
        } else {
            return Err(TokenizerError::UnknownToken);
        };

        let mut end_of_token = captures.get(0).unwrap().end();

        let ret = if captures.name("whitespace").is_some() {
            InternalToken::Whitespace
        } else if captures.name("badEofStringBody").is_some() || captures.name("clipped").is_some()
        {
            return Err(TokenizerError::UnexpectedEndOfFile);
        } else {
            InternalToken::PublicToken(if let Some(string) = captures.name("goodStringBody") {
                Token::TString(string.as_str())
            } else if let Some(block) = captures.name("block") {
                let block_char = block.as_str();
                if block_char == "(" {
                    Token::Block(Block::Start)
                } else if block_char == ")" {
                    Token::Block(Block::End)
                } else {
                    unreachable!()
                }
            } else if let Some(boolean) = captures.name("boolean") {
                end_of_token = boolean.end();
                let bool_str = boolean.as_str();
                if bool_str == "#t" {
                    Token::Bool(true)
                } else if bool_str == "#f" {
                    Token::Bool(false)
                } else {
                    unreachable!()
                }
            } else if let Some(symbol) = captures.name("symbol") {
                end_of_token = symbol.end();
                Token::Symbol(symbol.as_str())
            } else if let Some(number) = captures.name("number") {
                end_of_token = number.end();
                Token::Number(number.as_str())
            } else if let Some(dot) = captures.name("dot") {
                end_of_token = dot.end();
                Token::Dot
            } else if let Some(mark) = captures.name("mark") {
                if mark.as_str() == "'" {
                    Token::Mark(Mark::Quote)
                } else {
                    unreachable!()
                }
            } else {
                unreachable!()
            })
        };

        self.input = &self.input[end_of_token..];

        Ok(ret)
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<Token<'a>, TokenizerError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut unchecked_token;
        loop {
            unchecked_token = self.gen_token();
            if let Ok(ref token) = unchecked_token {
                //Grab another token if its whitespace
                if token.can_ignore() {
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if let Ok(InternalToken::EndOfFile) = unchecked_token {
            return None;
        }

        Some(unchecked_token.map(InternalToken::into_public))
    }
}
