// Tim Henderson <tim.tadh@gmail.com>
// Copyright 2014
// All rights reserved.
// For licensing information see the top level directory.


extern crate collections;

use self::collections::Vec;
use gram_lexer::*;

#[deriving(Show)]
pub struct Node {
    pub label : String,
    pub kids : Vec<Box<Node>>
}

#[deriving(Show)]
enum ParseError {
    Lex(LexError),
    NotImplemented,
    NoMoreInputExpected(TokenType),
    ExpectedButGot(TokenType,TokenType),
    UnconsumedInput(String)
}

impl Node {
    fn new(label : &str) -> Node {
        return Node{
            label: label.to_string(),
            kids: Vec::new()
        }
    }
    fn addkid(mut self, node : Node) -> Node {
        self.kids.push(box node);
        return self
    }
    fn enquekid(mut self, node : Node) -> Node {
        self.kids.insert(0, box node);
        self
    }
}

pub fn parse<'a, 'b>(lexer : &mut Lexer<'a>) -> Result<Node,ParseError> {
    let parser = match Parser::new(lexer) {
        Ok(p) => { p }
        Err(err) => { return Err(Lex(err)) }
    };
    parser.parse()
}

struct Parser<'a> {
    tokens : Vec<Token<'a>>
}

impl<'a> Parser<'a> {
    fn new<'a>(lexer : &mut Lexer<'a>) -> Result<Parser<'a>,LexError> {
        let mut tokens : Vec<Token<'a>> = Vec::new();
        for res in lexer {
            match res {
                Ok(tok) => {
                    tokens.push(tok)
                }
                Err(err) => {
                    return Err(err)
                }
            }
        }
        return Ok(Parser{ tokens : tokens })
    }

    fn parse(&self) -> Result<Node,ParseError> {
        let (i, node) = try!(self.Productions(0));
        if i != self.tokens.len() {
            return Err(UnconsumedInput(format!("{}", self.tokens.slice(i, self.tokens.len()))))
        }
        return Ok(node)
    }

    #[allow(non_snake_case)]
    fn Productions(&self, i : uint) -> Result<(uint,Node),ParseError> {
        self.epsilon(i, Node::new("Grammar"), |i| {
            let (a, prod) = try!(self.Production(i));
            let (b, list) = try!(self.Productions(a));
            return Ok((b, list.enquekid(prod)))
        })
    }

    #[allow(non_snake_case)]
    fn Production(&self, i : uint) -> Result<(uint,Node),ParseError> {
        let (a, nt) = try!(self.consume(i, NONTERM).and_then(|(a,tok)| {Ok((a,Node::new("NonTerm").addkid(Node::new(tok.lexeme))))}));
        let (b, _) = try!(self.consume(a, ARROW));
        let (c, body) = try!(self.Body(b));
        let (d, _) = try!(self.consume(c, SEMI));
        return Ok((d, Node::new("Production").addkid(nt).addkid(body)));
    }

    #[allow(non_snake_case)]
    fn Body(&self, i : uint) -> Result<(uint,Node),ParseError> {
        let (j, rule) = try!(self.Rule(i));
        let (k, body) = try!(self.Body_(j));
        return Ok((k, body.enquekid(rule)));
    }

    #[allow(non_snake_case)]
    fn Body_(&self, i : uint) -> Result<(uint,Node),ParseError> {
        self.epsilon(i, Node::new("Body"), |i| {
            let (j, _) = try!(self.consume(i, VBAR));
            let (k, rule) = try!(self.Rule(j));
            let (l, body) = try!(self.Body_(k));
            return Ok((l, body.enquekid(rule)))
        })
    }

    #[allow(non_snake_case)]
    fn Rule(&self, i : uint) -> Result<(uint,Node),ParseError> {
        self.epsilon(i, Node::new("Rule"), |i| {
            let (j, sym) = try!(self.Symbol(i));
            let (k, rule) = try!(self.Rule(j));
            return Ok((k, rule.enquekid(sym)))
        })
    }

    #[allow(non_snake_case)]
    fn Symbol(&self, i : uint) -> Result<(uint,Node),ParseError> {
        return self.consume(i, TERM).and_then(|(j, tok)| {
                    Ok((j, Node::new("Term").addkid(Node::new(tok.lexeme))))
                }).or(self.consume(i, NONTERM).and_then(|(j, tok)| {
                    Ok((j, Node::new("NonTerm").addkid(Node::new(tok.lexeme))))
                }).or(self.consume(i, EMPTY).and_then(|(j, tok)| {
                    Ok((j, Node::new("Empty").addkid(Node::new(tok.lexeme))))
                })));
    }

    fn epsilon<T, E>(&self, i : uint, n : T, f : |uint| -> Result<(uint,T),E>) -> Result<(uint,T),E> {
        return f(i).or(Ok((i,n)))
    }

    fn consume<'a>(&'a self, i : uint, toktype : TokenType) -> Result<(uint, Token<'a>), ParseError> {
        if i >= self.tokens.len() {
            return Err(NoMoreInputExpected(toktype));
        }
        let tok = self.tokens[i];
        return if tok.token == toktype {
                   Ok((i+1, tok))
               } else {
                   Err(ExpectedButGot(toktype, tok.token))
               };
    }
}

