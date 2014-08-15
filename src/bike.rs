#![crate_name = "bike"]
#![crate_type = "lib"]

use std::collections::{HashMap, TreeMap};
use std::io::{MemReader, IoError, IoErrorKind, EndOfFile};

#[deriving(PartialEq,Show,Clone)]
pub enum BObject {
    Number(f64),
    String(String),
    List(List),
    Dictonary(Dictonary),
}

pub type List = Vec<BObject>;
pub type Dictonary = TreeMap<String, BObject>;

pub enum ErrorCode {
}

#[deriving(PartialEq,Show)]
enum Token {
    LBrace,
    RBrace,
    LParen,
    RParen,
    Equals,
    StrTok(String),
    NumTok(f64),
    Ident(String),
    EOF,
}

struct Tokenizer {
    source: MemReader,
    ch: Option<char>,
}

#[deriving(PartialEq,Show)]
enum TokenError {
    IoError(IoError),
    InvalidNumber,
    DisclosedString,
    NotUtf8,
    UnexpectedT,
}

impl Tokenizer {
    fn new(src: Vec<u8>) -> Tokenizer {
        let mr = MemReader::new(src);
        Tokenizer{source: mr, ch: None}
    }

    fn token(&mut self) -> Result<Token, TokenError> {
        if self.ch == None {
            match self.source.read_char() {
                Ok(c) => self.ch = Some(c),
                Err(ref e) if e.kind == EndOfFile  => return Ok(EOF),
                Err(e) => return Err(IoError(e)),
            }
        }

        loop {
            return match self.ch.unwrap() { 
                ';' => {
                    try!(self.skip_line());
                    self.ch = match self.source.read_char() {
                        Ok(c) => Some(c),
                        Err(ref e) if e.kind == EndOfFile  => return Ok(EOF),
                        Err(e) => return Err(IoError(e)),
                    };
                    continue
                },
                c if c.is_whitespace() => {
                    self.ch = match self.source.read_char() {
                        Ok(c) => Some(c),
                        Err(ref e) if e.kind == EndOfFile  => return Ok(EOF),
                        Err(e) => return Err(IoError(e)),
                    };
                    continue
                },
                '{' => { self.ch = None ; Ok(LBrace) },
                '}' => { self.ch = None ; Ok(RBrace) },
                '(' => { self.ch = None ; Ok(LParen) },
                ')' => { self.ch = None ; Ok(RParen) },
                '=' => { self.ch = None ; Ok(Equals) },
                '-' | '0'..'9'       => self.num_token(),
                '\''                 => self.str_token(),
                c if c as u32 >= 65  => self.ident_token(),
                _ => Err(UnexpectedT),
            }
        }
    }
    fn skip_line(&mut self) -> Result<(), TokenError> {
        match self.source.read_line() {
            Ok(_) => Ok(()),
            Err(ref e) if e.kind == EndOfFile => Ok(()),
            Err(e) => Err(IoError(e)),
        }
    }

    fn num_token(&mut self) -> Result<Token, TokenError> {
        unimplemented!()
    }
    fn str_token(&mut self) -> Result<Token, TokenError> {
        let mut vec = Vec::new();

        loop {
            match self.source.read_until('\'' as u8) {
                Ok(v) => {
                    if v.last() != Some(&('\'' as u8)) {
                        return Err(DisclosedString)
                    };
                    vec = vec.append(v.as_slice());

                    match self.source.read_char() {
                        Ok('\'') => continue,
                        Ok(c) => {
                            self.ch = Some(c);
                            break
                        },
                        Err(ref e) if e.kind == EndOfFile  => {
                            self.ch = None;
                            break
                        },
                        Err(e) => return Err(IoError(e)),
                    }
                },
                Err(ref e) if e.kind == EndOfFile  => return Err(DisclosedString),
                Err(e) => return Err(IoError(e)),
            }
        }

        vec.pop().unwrap();
        match String::from_utf8(vec) {
            Ok(str) => Ok(StrTok(str)),
            Err(_) => Err(NotUtf8),
        }
    }
    fn ident_token(&mut self) -> Result<Token, TokenError> {
        fn reserved_char(c: char) -> bool {
            match c {
                ';' | '{' | '}' | '(' | ')' | '\'' | '=' => true,
                _ => false,
            }
        }

        let mut id = String::from_char(1, self.ch.unwrap());
        loop {
            match self.source.read_char() {
                Ok(c) if c.is_whitespace() || reserved_char(c)  => {
                    self.ch = Some(c);
                    break
                },
                Ok(c) => {
                    id.push_char(c);
                    continue
                },
                Err(ref e) if e.kind == EndOfFile  => {
                    self.ch = None;
                    break
                },
                Err(e) => return Err(IoError(e)),
            }
        };

        Ok(Ident(id))
    }
}


enum Nonterm {
    ListNT(List),
    DictNT(Dictonary),
    ObjectNT(BObject),
}

#[deriving(PartialEq,Show)]
enum ParseError {
//    IoError(IoError),
    TokenError(TokenError),
    UnexpectedEOF,
    ExpectedFound(Token, Token),
    Unexpected(Token),
    UnbalacedBracket(Token),
}

struct Parser {
    lexer: Tokenizer,
    token_stack: Vec<Token>,
    data_stack: Vec<Nonterm>,
}

impl Parser {
    fn new(lexer: Tokenizer) -> Parser {
        let ts = Vec::new();
        let ds = vec!(DictNT(TreeMap::new()));
        Parser{lexer: lexer, token_stack: ts, data_stack: ds}
    }

    fn parse(&mut self) -> Result<BObject, ParseError> {
        let mut head = try!(self.shift());

        loop {
            match head {
                EOF => {
                    if !self.token_stack.is_empty() { return Err(UnexpectedEOF) }
                    match self.data_stack.pop() {
                        Some(DictNT(d)) => return Ok(Dictonary(d)),
                        _ => unreachable!(),
                    }
                },
                StrTok(s) => self.data_stack.push(ObjectNT(String(s))),
                NumTok(n) => self.data_stack.push(ObjectNT(Number(n))),
                lb @ LBrace => {
                    self.token_stack.push(lb);
                    self.data_stack.push(DictNT(TreeMap::new()));
                },
                rb @ RBrace => {
                    let br = self.token_stack.pop();
                    match br {
                        Some(br @ LParen) => return Err(UnbalacedBracket(br)),
                        Some(_) => {},
                        None => return Err(UnbalacedBracket(rb)),
                    }
                    let obj = match self.data_stack.pop() {
                        Some(DictNT(dict)) => Dictonary(dict),
                        _ => unreachable!(),
                    };
                    self.data_stack.push(ObjectNT(obj));
                },
                lp @ LParen => {
                    self.token_stack.push(lp);
                    self.data_stack.push(ListNT(Vec::new()));
                },
                rp @ RParen => {
                    let br = self.token_stack.pop();
                    match br {
                        Some(br @ LBrace) => return Err(UnbalacedBracket(br)),
                        Some(_) => {},
                        None => return Err(UnbalacedBracket(rp)),
                    }
                    let obj = match self.data_stack.pop() {
                        Some(ListNT(lst)) => List(lst),
                        _ => unreachable!(),
                    };
                    self.data_stack.push(ObjectNT(obj));
                },
                _ => unimplemented!(),
            };

            match self.data_stack.pop() {
                Some(ObjectNT(obj)) => {
                    let mut lod = self.data_stack.pop().unwrap();
                    match lod {
                        ListNT(ref mut lst) => lst.push(obj),
                        DictNT(ref mut dict) => {
                            match try!(self.shift()) {
                                Equals => {},
                                tok => return Err(ExpectedFound(Equals, tok))
                            };
                            let ident = match try!(self.shift()) {
                                Ident(id) | StrTok(id) => id,
                                tok => return Err(Unexpected(tok)),
                            };
                            dict.insert(ident, obj);
                        },
                        _ => unimplemented!(),
                    };
                    self.data_stack.push(lod);
                },
                Some(data) => self.data_stack.push(data), // this is ugly, but mah movin' semantic
                None => unreachable!(),
            }

            head = try!(self.shift());
        }
    }

    fn shift(&mut self) -> Result<Token, ParseError> {
        match self.lexer.token() {
            Ok(t) => Ok(t),
            //Err(IoError(e)) => IoError(e),
            Err(e) => Err(TokenError(e)),
        }
    }
}


#[test] fn lexer_test() {
    let str = "('is it workin''?') = lol ; comment".as_bytes().into_vec();
    let mut lex = Tokenizer::new(str);

    assert_eq!(lex.token(), Ok(LParen));
    assert_eq!(lex.token(), Ok(StrTok("is it workin'?".into_string())));
    assert_eq!(lex.token(), Ok(RParen));
    assert_eq!(lex.token(), Ok(Equals));
    assert_eq!(lex.token(), Ok(Ident("lol".into_string())));
    assert_eq!(lex.token(), Ok(EOF));
}

#[test] fn parser_test() {
    let str = "'bikeML' = name 
                ('fun' 'minimalistic' 'crazy') = features 
                {'foo' = bar  'bar' = baz} = lol ; yeah!".as_bytes().into_vec();
    let mut parser = Parser::new(Tokenizer::new(str));
    let obj = parser.parse();

    let root = match obj {
        Ok(Dictonary(d)) => d,
        _ => fail!(),
    };

    assert_eq!(root["name".into_string()], String("bikeML".into_string()));

    let exp_lst = vec![
        String("fun".into_string()),
        String("minimalistic".into_string()),
        String("crazy".into_string()),
    ];
    assert_eq!(root["features".into_string()], List(exp_lst));

    let dict = match root["lol".into_string()].clone() {
        Dictonary(d) => d,
        _ => fail!(),
    };
    assert_eq!(dict["bar".into_string()], String("foo".into_string()));
    assert_eq!(dict["baz".into_string()], String("bar".into_string()));
}
