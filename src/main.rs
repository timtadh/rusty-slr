// Tim Henderson <tim.tadh@gmail.com>
// Copyright 2014
// All rights reserved.
// For licensing information see the top level directory.

#![feature(globs)]
#![feature(macro_rules)]

extern crate getopts;
extern crate libc;

use std::os;
use std::io;
use std::result::Result;

use gram_lexer::gram_lexer;
use gram_parser::parse;

mod gram_parser;
mod gram_lexer;
mod slr;

macro_rules! log(($fmt:expr$(, $msg:expr)*) => {
    (writeln![io::stderr(), $fmt $(, $msg)*]).ok().expect("log failed")
})

struct MainConfig<'a>{options : &'a [getopts::OptGroup]}

impl<'a> MainConfig<'a> {
    fn usage(&self) {
        let short_usage = "slr ...";
        log!("{}", getopts::usage(short_usage, self.options));
        unsafe { libc::exit(5); }
    }

    fn unwrap_or_die<T, E: std::fmt::Show>(&self, res : Result<T, E>) -> T {
        match res {
            Ok(stuff) => {
                stuff
            }
            Err(err) => {
                log!("{}", err);
                self.usage();
                fail!("unreachable");
            }
        }
    }

    fn read_file_or_die(&self, path : &str) -> String {
        self.unwrap_or_die(io::File::open(&Path::new(path))
          .read_to_end()
          .and_then(|bytes : Vec<u8>| -> Result<String,io::IoError> {
            Ok(self.unwrap_or_die(std::string::String::from_utf8(bytes)))
          }))
    }
}

fn main() {
    let cfg : MainConfig = MainConfig{options: &[
        getopts::optopt("g", "grammar", "the grammar to read", "<path>"),
        getopts::optflag("h", "help", "print this help menu")
    ]};


    let args: Vec<String> = os::args();

    let opts = cfg.unwrap_or_die(getopts::getopts(args.slice(1, args.len()), cfg.options));
    if opts.opt_present("h") {
        cfg.usage();
        return
    }

    let grammar_path = match opts.opt_str("g") {
        Some(s) => { s }
        None => { cfg.usage(); return }
    };

    log!("grammar path = {}", grammar_path);

    let grammar : String = cfg.read_file_or_die(grammar_path.as_slice());

    log!("the grammar from {} is {} characters long", grammar_path, grammar.len());
    let grammar = cfg.unwrap_or_die(parse(&mut gram_lexer(grammar.as_slice())));
    let grammar = slr::Grammar::new(grammar);
    let automaton = grammar.LR0_automaton();
    println!("{}", automaton);
    println!("\n");
    println!("{}", automaton.table());
}
