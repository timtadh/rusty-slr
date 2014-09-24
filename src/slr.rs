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

use gram_parser::Node;

#[deriving(Hash, Clone, Eq, PartialEq, PartialOrd, Show)]
pub enum Symbol {
    Term(String),
    NonTerm(String)
}

#[deriving(Hash, Eq, PartialEq, PartialOrd, Show)]
pub struct Production {
    nt : String,
    symbols : Vec<Symbol>
}


#[deriving(Hash, Clone, Eq, Show)]
pub struct Item<'a> {
    production : &'a Production,
    dot : uint
}

impl<'a> PartialEq for Item<'a> {
    fn eq<'a>(&self, other : &Item<'a>) -> bool {
        self.dot == other.dot && self.production.eq(other.production)
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

#[deriving(Show, Clone, Eq)]
pub struct ItemSet<'a> {
    items : Vec<Item<'a>>
}

impl<'a> ItemSet<'a> {
    pub fn new() -> ItemSet<'a> {
        return ItemSet{items : Vec::new()}
    }

    pub fn singleton(item : Item<'a>) -> ItemSet<'a> {
        let mut set = ItemSet::new();
        set.add(item);
        return set;
    }

    pub fn iter<'b>(&'b self) -> slice::Items<'b, Item<'a>> {
        return self.items.iter()
    }

    pub fn into_iter<'b>(self) -> vec::MoveItems<Item<'a>> {
        self.items.into_iter()
    }

    pub fn add<'b>(&'b mut self, item : Item<'a>) {
        let (idx, _) = self.find(&item);
        self.insert(idx, item);
    }

    fn insert<'b>(&'b mut self, i : uint, item : Item<'a>) {
        self.items.insert(i, item);
    }

    pub fn contains<'b>(&'b self, item : &Item<'a>) -> bool {
        let (_, has) = self.find(item);
        return has;
    }

    pub fn find<'b>(&'b self, item : &Item<'a>) -> (uint,bool) {
        let mut l : int = 0;
        let mut r : int = (self.items.len() as int) - 1;
        let mut m : int;
        while l <= r {
            m = ((r - l) >> 1) + l;
            if item < &self.items[m as uint] {
                r = m - 1;
            } else if item == &self.items[m as uint] {
                let mut j = m;
                while j >= 0 {
                    if j == 0 || item != &self.items[(j-1) as uint] {
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
}

impl<'a> Collection for ItemSet<'a> {
    fn len(&self) -> uint {
        return self.items.len()
    }

    fn is_empty(&self) -> bool {
        return self.items.is_empty()
    }
}

impl<'a> PartialEq for ItemSet<'a> {
    fn eq<'a>(&self, other : &ItemSet<'a>) -> bool {
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
    fn ne<'a>(&self, other : &ItemSet<'a>) -> bool {
        !self.eq(other)
    }
}

impl<'a> PartialOrd for ItemSet<'a> {
    fn partial_cmp(&self, other : &ItemSet<'a>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for ItemSet<'a> {
    fn cmp(&self, other : &ItemSet<'a>) -> Ordering {
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

#[deriving(Show)]
pub struct Grammar {
    start : String,
    symbols : HashSet<Symbol>,
    productions : HashMap<String, Vec<Production>>
}

#[deriving(Show)]
pub struct SLRState<'a> {
    id : uint,
    items : ItemSet<'a>,
    moves : HashMap<Symbol, uint>
}

#[deriving(Show)]
pub struct SLRAutomaton<'a> {
    grammar : &'a Grammar,
    states : Vec<SLRState<'a>>
}

impl Grammar {
    pub fn new(root : Node) -> Grammar {
        let mut symbols : HashSet<Symbol> = HashSet::new();
        let start = Grammar::name(Grammar::symbol(&*root.kids[0].kids[0]));
        symbols.insert(NonTerm(start.clone()));
        let mut productions : HashMap<String, Vec<Production>> = HashMap::new();
        for pnode in root.kids.iter() {
            println!("{}", pnode.kids[0])
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
        return Grammar{start:start, symbols: symbols, productions:productions}
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
        }
    }

    pub fn LR0_automaton<'a>(&'a self) -> SLRAutomaton<'a> {
        let mut A = SLRAutomaton{grammar: self, states: Vec::new()};
        let mut states : TreeMap<ItemSet<'a>,uint> = TreeMap::new();
        let mut stack : Vec<ItemSet<'a>> = Vec::new();
        let mut next_id : uint = 0;
        stack.push(self.closure(&self.start_items()));

        while stack.len() > 0 {
            let items = stack.pop().unwrap();
            states.insert(items.clone(), next_id);

            for sym in self.symbols.iter() {
                let next = self.goto(&items, sym);
                if next.len() == 0 {
                    continue;
                }
                if states.contains_key(&next) {
                    continue;
                }
                stack.push(next);
            }

            let state = SLRState{
                id : next_id,
                items : items,
                moves : HashMap::new()
            };
            next_id += 1;
            A.states.push(state);
        }

        for state in A.states.iter_mut() {
            for sym in self.symbols.iter() {
                let next = self.goto(&state.items, sym);
                if next.len() == 0 {
                    continue;
                }
                if states.contains_key(&next) {
                    let i = states[next];
                    state.moves.insert(sym.clone(), i);
                }
            }
        }
        return A;
    }

    fn start_items<'a>(&'a self) -> ItemSet<'a> {
        let mut items = ItemSet::new();
        for production in self.productions[self.start].iter() {
            items.add(Item{production: production, dot: 0});
        }
        return items;
    }

    pub fn closure<'a>(&'a self, items : &ItemSet<'a>) -> ItemSet<'a> {
        let mut ret = ItemSet::new();
        let mut stack : Vec<Item> = Vec::new();
        for item in items.iter() {
            stack.push((*item).clone());
        }
        while stack.len() > 0 {
            let item : Item = stack.pop().unwrap();
            if item.dot < item.production.symbols.len() {
                let ref sym : Symbol = item.production.symbols[item.dot];
                let prods = match *sym {
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
            ret.add(item);
        }
        return ret
    }

    #[allow(non_snake_case)]
    pub fn goto<'a>(&'a self, I : &ItemSet<'a>, X : &Symbol) -> ItemSet<'a> {
        let mut ret = ItemSet::new();
        for item in I.iter() {
            if item.dot >= item.production.symbols.len() {
                continue
            }
            let ref sym = item.production.symbols[item.dot];
            if sym != X {
                continue
            }
            let next = ItemSet::singleton(Item{production: item.production, dot: item.dot+1});
            for i in self.closure(&next).into_iter() {
                ret.add(i);
            }
        }
        return ret
    }
}

