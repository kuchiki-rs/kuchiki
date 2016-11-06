use html5ever::{QualName, LocalName};
use std::collections::hash_map::{self, HashMap};

/// Convenience wrapper around a hashmap that adds method for attributes in the null namespace.
#[derive(Debug, PartialEq, Clone)]
pub struct Attributes {
    /// A map of attributes whose name can have namespaces.
    pub map: HashMap<QualName, String>,
}

impl Attributes {
    /// Like HashMap::contains
    pub fn contains<A: Into<LocalName>>(&self, local_name: A) -> bool {
        self.map.contains_key(&QualName::new(ns!(), local_name.into()))
    }

    /// Like HashMap::get
    pub fn get<A: Into<LocalName>>(&self, local_name: A) -> Option<&str> {
        self.map.get(&QualName::new(ns!(), local_name.into())).map(AsRef::as_ref)
    }

    /// Like HashMap::get_mut
    pub fn get_mut<A: Into<LocalName>>(&mut self, local_name: A) -> Option<&mut String> {
        self.map.get_mut(&QualName::new(ns!(), local_name.into()))
    }

    /// Like HashMap::entry
    pub fn entry<A: Into<LocalName>>(&mut self, local_name: A) -> hash_map::Entry<QualName, String> {
        self.map.entry(QualName::new(ns!(), local_name.into()))
    }

    /// Like HashMap::insert
    pub fn insert<A: Into<LocalName>>(&mut self, local_name: A, value: String) -> Option<String> {
        self.map.insert(QualName::new(ns!(), local_name.into()), value)
    }

    /// Like HashMap::remove
    pub fn remove<A: Into<LocalName>>(&mut self, local_name: A) -> Option<String> {
        self.map.remove(&QualName::new(ns!(), local_name.into()))
    }
}
