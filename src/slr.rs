// Tim Henderson <tim.tadh@gmail.com>
// Copyright 2014
// All rights reserved.
// For licensing information see the top level directory.

extern crate collections;

use self::collections::Vec;
use std::collections::{HashMap,HashSet,TreeMap};
use std::slice;
use std::vec;
use std::cmp;
use std::fmt::{Formatter,Show,FormatError};
use std::cell::RefCell;

use gram_parser::Node;


fn find<T: Ord + Eq, S: Index<uint, T> + Collection>(items : &S, item : &T) -> (uint,bool) {
    let mut l : int = 0;
    let mut r : int = (items.len() as int) - 1;
    let mut m : int;
    while l <= r {
        m = ((r - l) >> 1) + l;
        if item < &(*items)[m as uint] {
            r = m - 1;
        } else if item == &(*items)[m as uint] {
            let mut j = m;
            while j >= 0 {
                if j == 0 || item != &(*items)[(j-1) as uint] {
                    return (j as uint, true)
                }
                j -= 1;
            }
        } else {
            l = m + 1;
        }
    }
    return (l as uint, false)
}

#[deriving(Hash, Clone, PartialEq, Ord, PartialOrd, Eq, Show)]
pub enum Symbol {
    Term(String),
    NonTerm(String),
    EmptyString
}

#[deriving(Show, Eq)]
pub struct SortedSet<T: Ord + Clone> {
    items : Vec<T>
}

impl<T: Ord + Clone> SortedSet<T> {
    pub fn new() -> SortedSet<T> {
        let mut items : Vec<T> = Vec::new();
        return SortedSet{items : items}
    }

    pub fn singleton(item : T) -> SortedSet<T> {
        let mut set : SortedSet<T> = SortedSet::new();
        set.add(item);
        return set;
    }

    pub fn iter<'b>(&'b self) -> slice::Items<'b, T> {
        return self.items.iter()
    }

    pub fn mut_iter<'b>(&'b mut self) -> slice::MutItems<'b, T> {
        self.items.mut_iter()
    }

    pub fn into_iter(self) -> vec::MoveItems<T> {
        self.items.into_iter()
    }

    pub fn add(&mut self, item : T) {
        let (idx, has) = find(&self.items, &item);
        if !has {
            self.insert(idx, item);
        }
    }

    pub fn addall(&mut self, items : SortedSet<T>) {
        for item in items.into_iter() {
            self.add(item)
        }
    }

    pub fn minus(&self, items : &SortedSet<T>) -> SortedSet<T> {
        let mut new : SortedSet<T> = SortedSet::new();
        for item in self.iter() {
            if !items.contains(item) {
                new.add((*item).clone());
            }
        }
        return new
    }

    fn insert(&mut self, i : uint, item : T) {
        self.items.insert(i, item);
    }

    pub fn contains<'b>(&'b self, item : &T) -> bool {
        let (_, has) = find(&self.items, item);
        return has;
    }
}

impl<T: Ord + Clone> Collection for SortedSet<T> {
    fn len(&self) -> uint {
        return self.items.len()
    }

    fn is_empty(&self) -> bool {
        return self.items.is_empty()
    }
}

impl<T: Ord + Clone> PartialEq for SortedSet<T> {
    fn eq(&self, other : &SortedSet<T>) -> bool {
        if self.items.len() != other.items.len() {
            return false;
        }
        for (a, b) in self.items.iter().zip(other.items.iter()) {
            if a != b {
                return false;
            }
        }
        return true
    }
    fn ne(&self, other : &SortedSet<T>) -> bool {
        !self.eq(other)
    }
}

impl<T: Ord + Clone> PartialOrd for SortedSet<T> {
    fn partial_cmp(&self, other : &SortedSet<T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Ord + Clone> Ord for SortedSet<T> {
    fn cmp(&self, other : &SortedSet<T>) -> Ordering {
        if self.items.len() < other.items.len() {
            return cmp::Less;
        } else if self.items.len() > other.items.len() {
            return cmp::Greater;
        }
        for (a, b) in self.items.iter().zip(other.items.iter()) {
            if a < b {
                return cmp::Less;
            } else if a > b {
                return cmp::Greater;
            }
        }
        return cmp::Equal;
    }
}

impl<T: Ord + Clone> Clone for SortedSet<T> {
    fn clone(&self) -> SortedSet<T> {
        let mut new : SortedSet<T> = SortedSet::new();
        for item in self.iter() {
            new.add((*item).clone());
        }
        return new
    }
}

#[deriving(Hash, Clone, PartialEq, Ord, PartialOrd, Eq, Show)]
pub struct Production {
    nt : String,
    symbols : Vec<Symbol>
}

impl Production {
    pub fn index_of(&self, symbol : &Symbol) -> Option<uint> {
        for (i, sym) in self.symbols.iter().enumerate() {
            if sym == symbol {
                return Some(i);
            }
        }
        None
    }
}

#[deriving(Hash, Eq)]
pub struct Item<'a> {
    production : &'a Production,
    dot : uint
}

impl<'a> Clone for Item<'a> {
    fn clone(&self) -> Item<'a> {
        return Item{
            production: self.production,
            dot: self.dot
        }
    }
}

impl<'a> Show for Item<'a> {
    fn fmt(&self, fmtr : &mut Formatter) -> Result<(), FormatError> {
        fmtr.write_str(format!("{} -> ", self.production.nt).as_slice()).ok();
        for (i, sym) in self.production.symbols.iter().enumerate() {
            if i == self.dot {
                fmtr.write_str(". ").ok();
            }
            fmtr.write_str(format!("{}", sym).as_slice()).ok();
            if i + 1 < self.production.symbols.len() {
                fmtr.write_str(" ").ok();
            }
        }
        if self.dot == self.production.symbols.len() {
                fmtr.write_str(" . ").ok();
        }
        return Ok(());
    }
}

impl<'a> PartialEq for Item<'a> {
    fn eq<'a>(&self, other : &Item<'a>) -> bool {
        self.dot == other.dot && *(self.production) == (*other.production)
    }
    fn ne<'a>(&self, other : &Item<'a>) -> bool {
        !self.eq(other)
    }
}

impl<'a> PartialOrd for Item<'a> {
    fn partial_cmp(&self, other : &Item<'a>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for Item<'a> {
    fn cmp(&self, other : &Item<'a>) -> Ordering {
        if self.production < other.production {
            cmp::Less
        } else if self.production == other.production {
            if self.dot < other.dot {
                cmp::Less
            } else if self.dot == other.dot {
                cmp::Equal
            } else {
                cmp::Greater
            }
        } else {
            cmp::Greater
        }
    }
}

pub struct Grammar {
    start : String,
    symbols : HashSet<Symbol>,
    first_cache : RefCell<HashMap<Symbol, SortedSet<Symbol>>>,
    productions : HashMap<String, Vec<Production>>
}

impl Grammar {
    pub fn new(root : Node) -> Grammar {
        let mut symbols : HashSet<Symbol> = HashSet::new();
        let start = Grammar::name(Grammar::symbol(&*root.kids[0].kids[0]));
        symbols.insert(NonTerm(start.clone()));
        let mut productions : HashMap<String, Vec<Production>> = HashMap::new();
        for pnode in root.kids.iter() {
            let nt : String = Grammar::name(Grammar::symbol(&*pnode.kids[0]));
            let mut bodies : Vec<Production> = productions.pop(&nt).unwrap_or(Vec::new());
            for rules in pnode.kids[1].kids.iter() {
                let mut body : Vec<Symbol> = Vec::new();
                for n in rules.kids.iter() {
                    let symbol = Grammar::symbol(&**n);
                    body.push(symbol.clone());
                    symbols.insert(symbol);
                }
                bodies.push(Production{nt: nt.clone(), symbols: body});
            }
            productions.insert(nt, bodies);
        }
        return Grammar{start:start, symbols: symbols, first_cache: RefCell::new(HashMap::new()), productions:productions}
    }

    fn symbol<'b>(node : &'b Node) -> Symbol {
        let name_node : &'b Node = &*node.kids[0];
        let name = &name_node.label;
        if node.label.as_slice() == "Term" {
            Term(name.clone())
        } else if node.label.as_slice() == "NonTerm" {
            NonTerm(name.clone())
        } else {
            fail!(format!("Unexpected Node {}", node))
        }
    }

    fn name(sym : Symbol) -> String {
        match sym {
            Term(s) => { s }
            NonTerm(s) => { s }
            EmptyString => { "".to_string() }
        }
    }

    #[allow(non_snake_case)]
    pub fn FIRST(&self, sym : Symbol) -> SortedSet<Symbol> {
        println!("FIRST {}", sym);
        if self.first_cache.borrow().contains_key(&sym) {
            return self.first_cache.borrow().deref()[sym].clone();
        }
        {
            let mut cache = self.first_cache.borrow_mut();
            cache.insert(sym.clone(), SortedSet::new());
        }
        let symbols = match sym.clone() {
            EmptyString => {
                SortedSet::singleton(sym.clone())
            }
            Term(_) => {
                SortedSet::singleton(sym.clone())
            }
            NonTerm(ref name) => {
                let mut symbols : SortedSet<Symbol> = SortedSet::new();
                for production in self.productions[name.clone()].clone().iter() {
                    let mut all_have_e = true;
                    for symbol in production.symbols.iter() {
                        if &sym == symbol {
                            continue;
                        }
                        let mut first = self.FIRST(symbol.clone());
                        symbols.addall(first.minus(&SortedSet::singleton(EmptyString)));
                        if first.contains(&EmptyString) {
                            all_have_e = false;
                            break;
                        }
                    }
                    if all_have_e {
                        symbols.add(EmptyString)
                    }
                }
                symbols
            }
            };
        let mut cache = self.first_cache.borrow_mut();
        cache.insert(sym.clone(), symbols.clone());
        symbols
    }

    #[allow(non_snake_case)]
    pub fn FIRST_vec(&self, syms : &Vec<Symbol>) -> SortedSet<Symbol> {
        let mut symbols : SortedSet<Symbol> = SortedSet::new();
        for sym in syms.iter() {
            let mut first = self.FIRST(sym.clone());
            symbols.addall(first.minus(&SortedSet::singleton(EmptyString)));
            if first.contains(&EmptyString) {
                return symbols;
            }
        }
        symbols.add(EmptyString);
        symbols
    }

    #[allow(non_snake_case)]
    pub fn FOLLOW(&self, nt : Symbol) -> SortedSet<Symbol> {
        println!("FOLLOW {}", nt);
        let mut symbols : SortedSet<Symbol> = SortedSet::new();
        match nt {
            NonTerm(ref name) => {
                for p in self.productions[name.clone()].iter() {
                    let i : uint = match p.index_of(&nt) {
                        Some(o) => { o }
                        None => { continue; }
                    };
                    if i + 1 < p.symbols.len() {
                        let mut first = self.FIRST_vec(&p.symbols);
                        if first.contains(&EmptyString) {
                            first = first.minus(&SortedSet::singleton(EmptyString));
                            if p.nt.as_slice() != name.as_slice() {
                                symbols.addall(self.FOLLOW(NonTerm(p.nt.clone())));
                            }
                        }
                        symbols.addall(first);
                    } else if i + 1 == p.symbols.len() && p.nt.as_slice() != name.as_slice() {
                        symbols.addall(self.FOLLOW(NonTerm(p.nt.clone())));
                    }
                }
            }
            _ => { fail!("Must pass in a NonTerm to FOLLOW"); }
        }
        symbols
    }


    #[allow(non_snake_case)]
    pub fn LR0_automaton<'a>(&'a self) -> SLRAutomaton<'a> {
        let mut A = SLRAutomaton{grammar: self, states: Vec::new()};
        let mut states : TreeMap<SortedSet<Item<'a>>,uint> = TreeMap::new();
        let mut stack : Vec<SortedSet<Item<'a>>> = Vec::new();
        let mut next_id : uint = 0;
        stack.push(self.closure(&self.start_items()));

        while stack.len() > 0 {
            let items = stack.pop().unwrap();

            for (_, next) in self.moves(&items).into_iter() {
                if states.contains_key(&next) {
                    continue;
                }
                stack.push(next);
            }

            if states.contains_key(&items) {
                continue;
            }

            states.insert(items.clone(), next_id);
            let state = SLRState{
                id : next_id,
                items : items,
                moves : HashMap::new()
            };
            next_id += 1;
            A.states.push(state);
        }

        for state in A.states.iter_mut() {
            for (sym, next) in self.moves(&state.items).into_iter() {
                if states.contains_key(&next) {
                    state.moves.insert(sym.clone(), states[next]);
                }
            }
        }
        return A;
    }

    fn start_items<'a>(&'a self) -> SortedSet<Item<'a>> {
        let mut items : SortedSet<Item<'a>> = SortedSet::new();
        for production in self.productions[self.start].iter() {
            items.add(Item{production: production, dot: 0});
        }
        return items;
    }

    pub fn closure<'a>(&'a self, items : &SortedSet<Item<'a>>) -> SortedSet<Item<'a>> {
        let mut ret : SortedSet<Item<'a>> = SortedSet::new();
        let mut stack : Vec<Item> = Vec::new();
        for item in items.iter() {
            stack.push((*item).clone());
        }
        while stack.len() > 0 {
            let item : Item = stack.pop().unwrap();
            ret.add(item.clone());
            if item.dot < item.production.symbols.len() {
                let ref sym : Symbol = item.production.symbols[item.dot];
                let prods = match *sym {
                        EmptyString => {
                            continue
                        }
                        Term(_) => {
                            continue
                        } NonTerm(ref name) => {
                            self.productions.find(name).unwrap()
                        }};
                for prod in prods.iter() {
                    let next_item = Item{production: prod, dot: 0};
                    if !ret.contains(&next_item) {
                        stack.push(next_item);
                    }
                }
            }
        }
        return ret
    }

    #[allow(non_snake_case)]
    pub fn moves<'a>(&'a self, I : &SortedSet<Item<'a>>) -> HashMap<Symbol, SortedSet<Item<'a>>> {
        let mut ret : HashMap<Symbol,SortedSet<Item<'a>>> = HashMap::new();
        for item in I.iter() {
            if item.dot >= item.production.symbols.len() {
                continue
            }
            let ref sym = item.production.symbols[item.dot];
            let next = SortedSet::singleton(Item{production: item.production, dot: item.dot+1});
            let mut items = ret.pop(sym).unwrap_or(SortedSet::new());
            for i in self.closure(&next).into_iter() {
                items.add(i)
            }
            ret.insert(sym.clone(), items);
        }
        return ret

    }
}

#[deriving(Show)]
pub struct SLRState<'a> {
    id : uint,
    items : SortedSet<Item<'a>>,
    moves : HashMap<Symbol, uint>
}

pub struct SLRAutomaton<'a> {
    grammar : &'a Grammar,
    states : Vec<SLRState<'a>>
}

#[deriving(Show)]
pub enum LRAction<'a> {
    Goto(uint),
    Shift(uint),
    Reduce(&'a Production),
    Accept,
    Error
}

#[deriving(Show)]
pub struct SLRTable<'a> {
    actions : HashMap<(uint,Symbol),LRAction<'a>>,
}

impl<'a> SLRAutomaton<'a> {
    pub fn table<'a>(&'a self) -> SLRTable<'a> {
        let mut table = SLRTable{
            actions : HashMap::new()
        };
        for (i,state) in self.states.iter().enumerate() {
            for (sym, target) in state.moves.iter() {
                match sym {
                    &EmptyString => {
                    }
                    &Term(_) => {
                        table.actions.insert((i,sym.clone()), Shift(*target));
                    }
                    &NonTerm(_) => {
                        table.actions.insert((i,sym.clone()), Goto(*target));
                    }
                };
            }
            for item in state.items.iter() {
                if item.dot == item.production.symbols.len() {
                    for sym in self.grammar.FOLLOW(NonTerm(item.production.nt.clone())).iter() {
                        table.actions.insert((i,sym.clone()), Reduce(item.production));
                    }
                }
            }
        }
        return table;
    }
}

impl<'a> Show for SLRAutomaton<'a> {
    fn fmt(&self, fmtr : &mut Formatter) -> Result<(), FormatError> {
        fmtr.write_str("LR(0) Automaton\n").ok();
        fmtr.write_str(format!("  symbols : {}\n", self.grammar.symbols).as_slice()).ok();
        for (i,state) in self.states.iter().enumerate() {
            fmtr.write_str(format!("    state : {}\n", i).as_slice()).ok();
            for item in state.items.iter() {
                fmtr.write_str(format!("      {}\n", item).as_slice()).ok();
            }
            fmtr.write_str("      moves : \n").ok();
            for (k,v) in state.moves.iter() {
                fmtr.write_str(format!("        {} -> {}\n", k, v).as_slice()).ok();
            }
        }
        return Ok(());
    }
}

