//! Allocation facts kept only for the addresses a query can see. Every
//! allocation adds facts about its fresh base address: nonzero, in range,
//! aligned, disjoint from each live object (two constants may overlap). A
//! path with dozens of objects carries hundreds of them, and they made
//! nearly every hash-table query slow although a query rarely mentions more
//! than one address. A fact is kept only when every base it names is
//! mentioned by the rest of the query (through `define-fun` bodies too);
//! kept facts go last, so the shared prefix of the incremental solver's
//! stack stays stable.
//!
//! Exactness: unsat of a subset is unsat of the whole. For sat, the dropped
//! facts each name a base nothing else mentions; such an object needs only
//! to be nonzero, aligned, in range and disjoint from the other objects. With
//! at most 2^20 objects of at most 2^32 bytes and alignment at most 2^12,
//! all objects together block less than 2^54 of the 2^64 addresses, so an
//! aligned free slot always exists and the model extends to every dropped
//! fact. A larger allocation turns slicing off for the rest of the run.

use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct Facts {
    /// Set once an allocation exceeds the placement bounds above.
    unbounded: bool,
    /// Objects at literal addresses: obstacles for placement, too.
    obstacles: usize,
    bases: HashSet<String>,
    /// The exact constraint texts allocation added, with the bases they name.
    facts: HashMap<String, Vec<String>>,
    bodies: HashMap<String, String>,
    /// Bases each defined name reaches.
    reaches: HashMap<String, Vec<String>>,
    /// Bases each other constraint text mentions.
    mentions: HashMap<String, Vec<String>>,
}

fn tokens(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| c == '(' || c == ')' || c.is_whitespace())
        .filter(|t| !t.is_empty())
}

impl Facts {
    /// A fresh allocation base; slicing stays exact only for bounded ones.
    pub fn base(&mut self, name: &str, size: u64, alignment: u64) {
        if size > 1 << 32 || alignment > 1 << 12 {
            self.unbounded = true;
        }
        self.bases.insert(name.to_owned());
        self.count();
    }
    /// Symbolic extents bounded by allocation's placement cap remain safe
    /// to slice unless their term depends on allocation addresses themselves.
    pub fn symbolic_extent(&mut self, term: &str) {
        if !self.mentions(term).is_empty() {
            self.unbounded = true;
        }
    }
    /// An object at a literal address (a dangling `NonNull`), which later
    /// allocations must not straddle.
    pub fn obstacle(&mut self) {
        self.obstacles += 1;
        self.count();
    }
    fn count(&mut self) {
        if self.bases.len() + self.obstacles > 1 << 20 {
            self.unbounded = true;
        }
    }
    /// A constraint allocation added about base addresses only.
    pub fn fact(&mut self, constraint: &str) {
        let bases = tokens(constraint)
            .filter(|t| self.bases.contains(*t))
            .map(str::to_owned)
            .collect();
        self.facts.insert(constraint.to_owned(), bases);
    }
    pub fn defined(&mut self, name: &str, body: &str) {
        self.bodies.insert(name.to_owned(), body.to_owned());
    }

    /// Bases a defined name reaches: an iterative post-order walk, so every
    /// child is settled before its parent.
    fn reaches(&mut self, name: &str) -> Vec<String> {
        let mut stack = vec![(name.to_owned(), false)];
        while let Some((current, expanded)) = stack.pop() {
            if self.reaches.contains_key(&current) {
                continue;
            }
            let body = self.bodies.get(&current).cloned().unwrap_or_default();
            if expanded {
                let mut found: Vec<String> = Vec::new();
                for t in tokens(&body) {
                    if self.bases.contains(t) {
                        found.push(t.to_owned());
                    } else if let Some(inner) = self.reaches.get(t) {
                        found.extend(inner.iter().cloned());
                    }
                }
                found.sort();
                found.dedup();
                self.reaches.insert(current, found);
                continue;
            }
            stack.push((current.clone(), true));
            for child in tokens(&body) {
                if self.bodies.contains_key(child) && !self.reaches.contains_key(child) {
                    stack.push((child.to_owned(), false));
                }
            }
        }
        self.reaches.get(name).cloned().unwrap_or_default()
    }

    /// Allocation bases a witness depends on: those `condition` names first,
    /// then those only a path constraint names, through `define-fun` bodies
    /// too. Allocation facts do not count; every object has them.
    pub fn placement_bases(&mut self, constraints: &[String], condition: &str) -> Vec<String> {
        let order = |a: &String, b: &String| (a.len(), a).cmp(&(b.len(), b));
        let mut found = self.mentions(condition);
        found.sort_by(order);
        let mut path = Vec::new();
        for constraint in constraints {
            if !self.facts.contains_key(constraint) {
                path.extend(self.mentions(constraint));
            }
        }
        path.sort_by(order);
        path.dedup();
        path.retain(|b| !found.contains(b));
        found.extend(path);
        found
    }

    fn mentions(&mut self, constraint: &str) -> Vec<String> {
        if let Some(known) = self.mentions.get(constraint) {
            return known.clone();
        }
        let symbols: Vec<String> = tokens(constraint)
            .filter(|t| self.bases.contains(*t) || self.bodies.contains_key(*t))
            .map(str::to_owned)
            .collect();
        let mut found = Vec::new();
        for symbol in symbols {
            if self.bases.contains(&symbol) {
                found.push(symbol);
            } else {
                found.extend(self.reaches(&symbol));
            }
        }
        found.sort();
        found.dedup();
        self.mentions.insert(constraint.to_owned(), found.clone());
        found
    }

    /// The constraints to assert for `extra`: every non-fact in order, then
    /// the facts whose bases the others all mention. `None` when allocation
    /// added no fact.
    pub fn slice(&mut self, constraints: &[String], extra: &str) -> Option<Vec<String>> {
        if self.facts.is_empty() || self.unbounded {
            return None;
        }
        let mut seen: HashSet<String> = self.mentions(extra).into_iter().collect();
        let mut kept = Vec::new();
        let mut facts = Vec::new();
        for constraint in constraints {
            if self.facts.contains_key(constraint) {
                facts.push(constraint);
            } else {
                seen.extend(self.mentions(constraint));
                kept.push(constraint.clone());
            }
        }
        if facts.is_empty() {
            return None;
        }
        for fact in facts {
            if self.facts[fact].iter().all(|b| seen.contains(b)) {
                kept.push(fact.clone());
            }
        }
        Some(kept)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocation_facts_follow_the_addresses_a_query_sees() {
        let mut facts = Facts::default();
        facts.base("v1", 8, 8);
        facts.base("v2", 8, 8);
        facts.defined("d5", "(bvadd v1 (_ bv8 64))");
        // d6 names d5 directly and through d7: the shared child is settled
        // before both parents.
        facts.defined("d7", "(bvmul d5 (_ bv2 64))");
        facts.defined("d6", "(bvadd d7 d5 x)");
        let own = "(not (= v1 (_ bv0 64)))".to_string();
        let apart =
            "(or (bvule (bvadd v1 (_ bv8 64)) v2) (bvule (bvadd v2 (_ bv8 64)) v1))".to_string();
        let path = vec![own.clone(), apart.clone(), "(= x (_ bv3 64))".to_string()];
        facts.fact(&own);
        facts.fact(&apart);
        let plain = vec![path[2].clone()];
        // No address seen: both facts go.
        assert_eq!(
            facts.slice(&path, "(bvult x (_ bv9 64))"),
            Some(plain.clone())
        );
        // v1 seen through two definitions: its own fact, not the pair.
        let mut one = plain.clone();
        one.push(own.clone());
        assert_eq!(facts.slice(&path, "(= d6 (_ bv0 64))"), Some(one));
        // Both bases seen: every fact, after the other constraints.
        let mut both = path.clone();
        both.push("(= v2 y)".into());
        let mut all = vec![
            path[2].clone(),
            "(= v2 y)".to_string(),
            own.clone(),
            apart.clone(),
        ];
        assert_eq!(facts.slice(&both, "(= v1 z)"), Some(all.clone()));
        // An obligation about an address is never itself dropped.
        all.truncate(1);
        all.push(own);
        assert_eq!(
            facts.slice(&path, "(not (= (bvurem v1 (_ bv4 64)) (_ bv0 64)))"),
            Some(all)
        );
    }

    #[test]
    fn address_dependent_symbolic_extents_disable_slicing() {
        let mut facts = Facts::default();
        facts.base("base", 8, 8);
        facts.defined("length", "(bvadd (bvand base (_ bv63 64)) (_ bv1 64))");
        facts.symbolic_extent("length");
        let condition = "(not (= base (_ bv0 64)))".to_owned();
        facts.fact(&condition);
        assert_eq!(facts.slice(&[condition], "true"), None);
    }

    #[test]
    fn huge_allocations_turn_slicing_off() {
        let mut facts = Facts::default();
        facts.base("v1", 1, 1 << 63);
        let fact = "(not (= v1 (_ bv0 64)))".to_string();
        facts.fact(&fact);
        // Placement is no longer guaranteed, so nothing is dropped.
        assert_eq!(facts.slice(std::slice::from_ref(&fact), "true"), None);
        // Likewise for a huge size.
        let mut facts = Facts::default();
        facts.base("v1", (1 << 32) + 1, 8);
        facts.fact(&fact);
        assert_eq!(facts.slice(&[fact], "true"), None);
    }
}
