use html5ever::{QualName, LocalName};
use std::collections::hash_map::{self, HashMap};

/// Convenience wrapper around a hashmap that adds method for attributes in the null namespace.
#[derive(Debug, PartialEq, Clone)]
pub struct Attributes {
    /// A map of attributes whose name can have namespaces.
    pub map: HashMap<QualName, String>,
}

macro_rules! namespaceless {
    ($local_name: expr) => {
        QualName::new(None, ns!(), $local_name.into())
    }
}

impl Attributes {
    /// Like HashMap::contains
    pub fn contains<A: Into<LocalName>>(&self, local_name: A) -> bool {
        self.map.contains_key(&namespaceless!(local_name))
    }

    /// Like HashMap::get
    pub fn get<A: Into<LocalName>>(&self, local_name: A) -> Option<&str> {
        self.map.get(&namespaceless!(local_name)).map(AsRef::as_ref)
    }

    /// Like HashMap::get_mut
    pub fn get_mut<A: Into<LocalName>>(&mut self, local_name: A) -> Option<&mut String> {
        self.map.get_mut(&namespaceless!(local_name))
    }

    /// Like HashMap::entry
    pub fn entry<A: Into<LocalName>>(&mut self, local_name: A) -> hash_map::Entry<QualName, String> {
        self.map.entry(namespaceless!(local_name))
    }

    /// Like HashMap::insert
    pub fn insert<A: Into<LocalName>>(&mut self, local_name: A, value: String) -> Option<String> {
        self.map.insert(namespaceless!(local_name), value)
    }

    /// Like HashMap::remove
    pub fn remove<A: Into<LocalName>>(&mut self, local_name: A) -> Option<String> {
        self.map.remove(&namespaceless!(local_name))
    }
}
