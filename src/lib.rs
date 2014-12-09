//! A generic [Markov chain](https://en.wikipedia.org/wiki/Markov_chain) for almost any type. This 
//! uses HashMaps internally, and so Eq and Hash are both required.
//!
//! # Examples
//!
//! ```
//! use markov::Chain;
//! 
//! let mut chain = Chain::new("START".into_string(), "END".into_string());
//! chain.feed_str("I like cats and I like dogs.");
//! println!("{}", chain.generate_str());
//! ```
#![feature(slicing_syntax)]
#![warn(missing_docs)]

use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::hash::Hash;
use std::io::{BufferedReader, File};
use std::rand::{Rng, task_rng};
use std::rc::Rc;

/// A generic [Markov chain](https://en.wikipedia.org/wiki/Markov_chain) for almost any type. This 
/// uses HashMaps internally, and so Eq and Hash are both required.
pub struct Chain<T: Eq + Hash> {
    map: HashMap<Rc<T>, HashMap<Rc<T>, uint>>,
    start: Rc<T>,
    end: Rc<T>,
}

impl<T: Eq + Hash> Chain<T> {
    /// Constructs a new Markov chain using the given tokens as the marked starting and ending
    /// points for generation.
    pub fn new(start: T, end: T) -> Chain<T> {
        let start = Rc::new(start);
        let end = Rc::new(end);
        Chain {
            map: {
                let mut map = HashMap::new();
                map.insert(start.clone(), HashMap::new());
                map.insert(end.clone(), HashMap::new());
                map
            },
            start: start, end: end
        }
    }

    /// Feeds the chain a collection of tokens. This operation is O(n) where n is the number of
    /// tokens to be fed into the chain.
    pub fn feed(&mut self, tokens: Vec<T>) -> &mut Chain<T> {
        if tokens.len() == 0 { return self }
        let mut toks = Vec::new();
        toks.push(self.start.clone());
        toks.extend(tokens.into_iter().map(|token| {
            let rc = Rc::new(token);
            if !self.map.contains_key(&rc) {
                self.map.insert(rc.clone(), HashMap::new());
            }
            rc
        }));
        toks.push(self.end.clone());
        for p in toks.windows(2) {
            self.map[p[0]].add(p[1].clone());
        }
        self
    }

    /// Generates a collection of tokens from the chain. This operation is O(mn) where m is the
    /// length of the generated collection, and n is the number of possible states from a given
    /// state.
    pub fn generate(&self) -> Vec<Rc<T>> {
        let mut ret = Vec::new();
        let mut curs = self.start.clone();
        while curs != self.end {
            curs = self.map[curs].next();
            ret.push(curs.clone());
        }
        ret.pop();
        ret
    }

    /// Generates a collection of tokens from the chain, starting with the given token. This
    /// operation is O(mn) where m is the length of the generated collection, and n is the number
    /// of possible states from a given state.
    pub fn generate_from_token(&self, token: T) -> Vec<Rc<T>> {
        let token = Rc::new(token);
        let mut ret = vec![token.clone()];
        let mut curs = token;
        while curs != self.end {
            curs = self.map[curs].next();
            ret.push(curs.clone());
        }
        ret.pop();
        ret
    }
}

impl Chain<String> {
    /// Creates a new Chain intended specifically for strings. This uses the Unicode start of text
    /// and end of text control characters as the starting and ending tokens for the chain.
    pub fn for_strings() -> Chain<String> {
        Chain::new("\u0002".into_string(), "\u0003".into_string())
    }

    /// Feeds a string of text into the chain. This string should omit ending punctuation.
    pub fn feed_str(&mut self, string: &str) -> &mut Chain<String> {
        self.feed(string.split_str(" ").map(|s| s.into_string()).collect())
    }

    /// Feeds a properly formatted file into the chain. This file should be formatted such that
    /// each line is a new sentence. Periods, exclamation points, and question marks should be 
    /// excluded from the ends of each line.
    pub fn feed_file(&mut self, path: &Path) -> &mut Chain<String> {
        let mut reader = BufferedReader::new(File::open(path));
        for line in reader.lines() {
            let line = line.unwrap();
            let words: Vec<_> = line.split([' ', '\t', '\n', '\r'][])
                                    .filter(|word| !word.is_empty())
                                    .collect();
            self.feed(words.iter().map(|s| s.into_string()).collect());
        }
        self
    }

    /// Generates a random string of text.
    pub fn generate_str(&self) -> String {
        let vec = self.generate();
        let mut ret = String::new();
        for s in vec.iter() {
            ret.push_str(s[]);
            ret.push_str(" ");
        }
        let len = ret.len();
        ret.truncate(len - 1);
        ret.push_str(".");
        ret
    }

    /// Generates a random string of text starting with the desired token.
    pub fn generate_str_from_token(&self, string: &str) -> String {
        let vec = self.generate_from_token(string.into_string());
        let mut ret = String::new();
        for s in vec.iter() {
            ret.push_str(s[]);
            ret.push_str(" ");
        }
        let len = ret.len();
        ret.truncate(len - 1);
        ret.push_str(".");
        ret
    }
}

/// A collection of states for the Markov chain.
trait States<T: PartialEq> {
    /// Adds a state to this states collection.
    fn add(&mut self, token: Rc<T>);
    /// Gets the next state from this collection of states.
    fn next(&self) -> Rc<T>;
}

impl<T: Eq + Hash> States<T> for HashMap<Rc<T>, uint> {
    fn add(&mut self, token: Rc<T>) {
        match self.entry(token) {
            Occupied(mut e) => *e.get_mut() += 1,
            Vacant(e) => { e.set(1); },
        }
    }

    fn next(&self) -> Rc<T> {
        let mut sum = 0;
        for &value in self.values() {
            sum += value;
        }
        let mut rng = task_rng();
        let cap = rng.gen_range(0, sum);
        sum = 0;
        for (key, &value) in self.iter() {
            sum += value;
            if sum > cap {
                return key.clone()
            }
        }
        unreachable!("The random number generator failed.")
    }
}

#[cfg(test)]
mod test {
    use super::Chain;

    #[test]
    fn new() {
        Chain::new(0u, 100u);
        Chain::for_strings();
    }

    #[test]
    fn feed() {
        let mut chain = Chain::new(0u, 100u);
        chain.feed(vec![3u, 5u, 10u]).feed(vec![5u, 12u]);
    }

    #[test]
    fn generate() {
        let mut chain = Chain::new(0u, 100u);
        chain.feed(vec![3u, 5u, 10u]).feed(vec![5u, 12u]);
        let v = chain.generate().map_in_place(|v| *v);
        assert!([vec![3u, 5u, 10u], vec![3u, 5u, 12u], vec![5u, 10u], vec![5u, 12u]].contains(&v));
    }

    #[test]
    fn generate_from_token() {
        let mut chain = Chain::new(0u, 100u);
        chain.feed(vec![3u, 5u, 10u]).feed(vec![5u, 12u]);
        let v = chain.generate_from_token(5u).map_in_place(|v| *v);
        assert!([vec![5u, 10u], vec![5u, 12u]].contains(&v));
    }

    #[test]
    fn feed_str() {
        let mut chain = Chain::for_strings();
        chain.feed_str("I like cats and dogs");
    }

    #[test]
    fn generate_str() {
        let mut chain = Chain::for_strings();
        chain.feed_str("I like cats").feed_str("I hate cats");
        assert!(["I like cats.", "I hate cats."].contains(&chain.generate_str()[]));
    }

    #[test]
    fn generate_str_from_token() {
        let mut chain = Chain::for_strings();
        chain.feed_str("I like cats").feed_str("cats are cute");
        assert!(["cats.", "cats are cute."].contains(&chain.generate_str_from_token("cats")[]));
    }
}