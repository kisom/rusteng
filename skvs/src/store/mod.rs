//! store implements the backing key-value store for the simple
//! key-value store. At its core, it is a hash map linking a `String`
//! key to an `Entry`.
pub mod entry;

use self::entry::Entry;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::string::ToString;

#[derive(Clone, Copy, Debug, PartialEq)]
/// Result contains results for write operations on the SKVS.
pub enum WriteResult {
    /// AlreadyExists is returned when inserting an entry under a key
    /// that already exists. It implies that the insert was rejected.
    AlreadyExists,
    /// Inserted is returned when inserting an entry under a key that
    /// didn't already exist.
    Inserted,
    /// Updated is returned when updating the entry for an
    /// already-existing key.
    Updated,
}

use self::WriteResult::*;

impl ToString for WriteResult {
    fn to_string(&self) -> String {
        match *self {
            AlreadyExists => return "key already exists".to_string(),
            Inserted      => return "new entry inserted".to_string(),
            Updated       => return "updated existing entry".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
/// metrics contains information about the SKVS.
pub struct Metrics {
    /// last_update stores the timestamp for the last time the store
    /// was updated; a call to `Store::delete` or `Store::add` will trigger this.
    pub last_update: i64,

    /// last_write stores the timestamp for the last time the store
    /// was written to disk.
    pub last_write: i64,

    /// size stores the current number of keys in the store.
    pub size: usize,

    /// write_error contains a string error message indicating why the
    /// last write failed. If the last write was successful, this
    /// field will be empty.
    pub write_error: String,    
}

impl Metrics {
    /// new returns initialises an empty Metrics structure.
    pub fn new() -> Metrics {
        Metrics { last_update: 0, last_write: 0, size: 0, write_error: "".to_string() }
    }
}

/// A `Store` is a simple key value store that persists to disk.
pub struct Store {
    /// path is the location on disk of the persisted SKVS.
    pub path: String,

    pub metrics: Metrics,
    pub values: HashMap<String, Entry>,
}

/// `new` returns an empty `Store`.
pub fn new(store_path: String) -> Store {
    Store {
        path: store_path.clone(),
        metrics: Metrics::new(),
        values: HashMap::new(),
    }
}

impl Store {
    /// len returns the number of entries in the key-value store.
    fn len(&self) -> usize {
        self.values.len()
    }

    /// insert writes a new entry. The expectation is that the entry doesn't
    /// exist; if it does, `AlreadyExists` is returned. Otherwise, the entry
    /// is inserted and `Inserted` is returned.
    fn insert(&mut self, k: String, v: String) -> WriteResult {
        if self.values.contains_key(&k) {
            AlreadyExists
        } else {
            self.values.insert(k, Entry::from_string(v));
            Inserted
        }                
    }

    /// update changes the value for `k` to `v`. If there was no
    /// existing entry for `k`, `Inserted` is returned. Otherwise,
    /// `Updated` is returned. Note that if `v` is the same as the
    /// existing value, the entry will not be changed but `Updated` is
    /// still returned.
    fn update(&mut self, k: String, v: String) -> WriteResult {
        // TODO(kyle): return AlreadyExists if v == old.value.
        //
        // pretty sure this function is an abomination.
        let wr: WriteResult;
        let old: Option<Entry>;
        let mut tmp_values = self.values.clone();

        match tmp_values.entry(k.clone()) {
            Occupied(e) => {
                old = Some(e.get().clone());
                wr = Updated;
                
            },
            Vacant(_)   => {
                old = None;
                wr = Inserted;
            }
        }

        match old {
            Some(ref ent) => {
                self.values.insert(k, Entry::update_from_string(ent, v));
            },
            None          => {
                self.values.insert(k, Entry::from_string(v));
            }
        }

        return wr;
    }

    /// get returns Some(value) if the key is present in the SKVS.
    fn get(&mut self, k: String) -> Option<String> {
        match self.values.entry(k.clone()) {
            Occupied(ent) => return Some(ent.get().value.clone()),
            Vacant(_)     => return None,
        }
    }
}


#[test]
fn test_store() {
    let mut kvs = new("".to_string());
    assert_eq!(kvs.len(), 0);

    let mut wr: WriteResult;
    wr = kvs.insert("X-Pro2".to_string(), "Fujifilm".to_string());
    assert_eq!(wr, Inserted);
    assert_eq!(kvs.len(), 1);

    // Make a mistake.
    wr = kvs.insert("D800".to_string(), "Canon".to_string());
    assert_eq!(wr, Inserted);
    assert_eq!(kvs.len(), 2);

    // Fix it.
    wr = kvs.insert("D800".to_string(), "Nikon".to_string());
    assert_eq!(wr, AlreadyExists);
    assert_eq!(kvs.len(), 2);

    wr = kvs.update("D800".to_string(), "Nikon".to_string());
    assert_eq!(wr, Updated);
    assert_eq!(kvs.len(), 2);

    let mut v = kvs.get("D800".to_string());
    assert_eq!(v.expect("missing entry"), "Nikon".to_string());

    v = kvs.get("X-Pro2".to_string());
    assert_eq!(v.expect("missing entry"), "Fujifilm".to_string());

    v = kvs.get("EOS 5D Mark II".to_string());
    assert!(v.is_none());
}
