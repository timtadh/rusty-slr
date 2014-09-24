// Tim Henderson <tim.tadh@gmail.com>
// Copyright 2014
// All rights reserved.
// For licensing information see the top level directory.

use std::iter::Iterator;

#[deriving(Show)]
#[deriving(PartialEq)]
pub enum TokenType {
    TERM,
    NONTERM,
    SEMI,
    VBAR,
    ARROW,
    EMPTY
}

#[deriving(Show)]
pub enum LexError {
    UnexpectedCharacter(char),
    BadState(uint),

}

#[deriving(Show)]
pub struct Token<'a> {
    pub token : TokenType,
    pub lexeme : &'a str,
}

#[deriving(Show)]
pub struct Lexer<'a> {
    text : &'a str,
    tc : uint,
    failed : bool
}

pub fn gram_lexer<'a>(text : &'a str) -> Lexer<'a> {
    return Lexer{
        text: text,
        tc: 0,
        failed: false,
    };
}

impl<'a> Lexer<'a> {
    fn white(ch : char) -> bool {
        ch == ' ' || ch == '\n' || ch == '\t'
    }

    fn big(ch : char) -> bool {
        'A' <= ch && ch <= 'Z'
    }

    fn not_big(ch : char) -> bool {
        ('a' <= ch && ch <= 'z') || ('0' <= ch && ch <= '9') || ch == '_' || ch == '\''
    }

    fn alpha_num(ch : char) -> bool {
        ('a' <= ch && ch <= 'z') || ('A' <= ch && ch <= 'Z') || ('0' <= ch && ch <= '9') || ch == '_' || ch == '\''
    }
}

impl<'a> Iterator<Result<Token<'a>,LexError>> for Lexer<'a> {
    fn next(&mut self) -> Option<Result<Token<'a>,LexError>> {
        if self.failed {
            return None
        }
        let text : &str = self.text;
        let mut state = 1;
        let mut start_tc = self.tc;
        let mut tc = self.tc;
        while tc < self.text.len() {
            let ch_range = text.char_range_at(tc);
            let ch = ch_range.ch;
            let mut next_tc = ch_range.next;
            state = match state {
                1 => {
                    if Lexer::white(ch) {
                        2
                    } else if ch == ';' {
                        3
                    } else if ch == '-' {
                        4
                    } else if ch == 'e' {
                        6
                    } else if ch == '|' {
                        9
                    } else if Lexer::big(ch) {
                        7
                    } else {
                        self.failed = true;
                        return Some(Err(UnexpectedCharacter(ch)))
                    }
                } 2 => {
                    if Lexer::white(ch) {
                        2
                    } else {
                        next_tc = tc;
                        start_tc = tc;
                        1
                    }
                } 3 => {
                    self.tc = next_tc;
                    return Some(Ok(Token{token:SEMI,lexeme:text.slice(start_tc,tc)}))
                } 4 => {
                    if ch == '>' {
                        5
                    } else {
                        return Some(Err(UnexpectedCharacter(ch)))
                    }
                } 5 => {
                    self.tc = next_tc;
                    return Some(Ok(Token{token:ARROW,lexeme:text.slice(start_tc,tc)}))
                } 6 => {
                    self.tc = next_tc;
                    return Some(Ok(Token{token:EMPTY,lexeme:text.slice(start_tc,tc)}))
                } 7 => {
                    if Lexer::big(ch) {
                        10
                    } else if Lexer::not_big(ch) {
                        8
                    } else {
                        self.tc = next_tc;
                        return Some(Ok(Token{token:NONTERM,lexeme:text.slice(start_tc,tc)}))
                    }
                } 8 => {
                    if Lexer::alpha_num(ch) {
                        8
                    } else {
                        self.tc = next_tc;
                        return Some(Ok(Token{token:NONTERM,lexeme:text.slice(start_tc,tc)}))
                    }
                } 9 => {
                    self.tc = next_tc;
                    return Some(Ok(Token{token:VBAR,lexeme:text.slice(start_tc,tc)}))
                } 10 => {
                    if Lexer::big(ch) {
                        10
                    } else if Lexer::not_big(ch) {
                        8
                    } else {
                        self.tc = next_tc;
                        return Some(Ok(Token{token:TERM,lexeme:text.slice(start_tc,tc)}))
                    }
                } _ => {
                    self.failed = true;
                    return Some(Err(BadState(state)))
                }
            };
            tc = next_tc;
        }
        None
    }
}

