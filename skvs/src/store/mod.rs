//! store implements the backing key-value store for the simple
//! key-value store. At its core, it is a hash map linking a `String`
//! key to an `Entry`.
pub mod entry;

extern crate serde;
extern crate serde_json;
extern crate time;

use self::entry::Entry;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::error::Error;
use std::fs::File;
use std::io;
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
    /// already-existing key; this occurs both with `update`
    /// and `delete`.
    Updated,
    /// DoesNotExist is returned when deleting a key that doesn't
    /// exist.
    DoesNotExist,
}

use self::WriteResult::*;

impl ToString for WriteResult {
    fn to_string(&self) -> String {
        match *self {
            AlreadyExists => return "key already exists".to_string(),
            Inserted      => return "new entry inserted".to_string(),
            Updated       => return "entry was updated".to_string(),
            DoesNotExist  => return "key doesn't exist".to_string(),
        }
    }
}

/// metrics contains information about the SKVS.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Metrics {
    /// last_update stores the timestamp for the last time the store
    /// was updated; a call to insert, update, or delete will update
    /// this field.
    pub last_update: i64,

    /// last_write stores the timestamp for the last time the store
    /// was written to disk.
    pub last_write: i64,

    /// size stores the current number of keys in the store.
    pub size: usize,
}

impl Metrics {
    /// new returns initialises an empty Metrics structure.
    pub fn new() -> Metrics {
        Metrics { last_update: 0, last_write: 0, size: 0 }
    }
}

/// A `Store` is a simple key value store that persists to disk.
#[derive(Clone, Debug, Serialize, Deserialize)]
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
    pub fn load(path: String) -> Result<Store, io::Error> {
        let file = File::open(path.clone())?;
        match serde_json::from_reader(file) {
            Ok(store) => Ok(store),
            Err(err)  => Err(io::Error::new(io::ErrorKind::Other, err.description())),
        }
    }

    /// `flush` writes the store to disk.
    pub fn flush(&mut self) -> Result<(), io::Error> {
        if self.path == "" {
            return Ok(());
        }
        self.update_metrics(false, true);
        
        let file = File::create(self.path.clone())?;
        match serde_json::to_writer(file, self) {
            Ok(_)    => Ok(()),
            Err(err) => Err(io::Error::new(io::ErrorKind::Other, err.description())),
        }
    }
    
    /// `update_metrics` makes sure the metrics field is up to
    /// date. if `write` is true, the `last_update` field is set to
    /// the current time stamp and the `size` field is set to the
    /// current HashMap size. If `persist` is true, the `last_write`
    /// field is updated.
    fn update_metrics(&mut self, write: bool, persist: bool) {
        let mut metrics = self.metrics;

        if write {
            metrics.last_update = time::get_time().sec;
            metrics.size = self.len();
        }

        if persist {
            metrics.last_write = time::get_time().sec;
        }

        self.metrics = metrics;
    }

    /// len returns the number of entries in the key-value store.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// insert writes a new entry. The expectation is that the entry doesn't
    /// exist; if it does, `AlreadyExists` is returned. Otherwise, the entry
    /// is inserted and `Inserted` is returned.
    pub fn insert(&mut self, k: String, v: String) -> WriteResult {
        if self.values.contains_key(&k) {
            AlreadyExists
        } else {
            self.values.insert(k, Entry::from_string(v));
            self.update_metrics(true, false);
            Inserted
        }
    }

    /// update changes the value for `k` to `v`. If there was no
    /// existing entry for `k`, `Inserted` is returned. Otherwise,
    /// `Updated` is returned. Note that if `v` is the same as the
    /// existing value, the entry will not be changed but `Updated` is
    /// still returned.
    pub fn update(&mut self, k: String, v: String) -> WriteResult {
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

        self.update_metrics(true, false);
        return wr;
    }

    /// `get` returns `Some(value)` if the key is present in the SKVS.
    pub fn get(&mut self, k: String) -> Option<String> {
        match self.values.entry(k.clone()) {
            Occupied(ent) => return Some(ent.get().value.clone()),
            Vacant(_)     => return None,
        }
    }

    /// `delete` removes the key from the database.
    pub fn delete(&mut self, k: String) -> WriteResult {
        if self.values.contains_key(&k) {
            self.values.remove(&k);
            self.update_metrics(true, false);
            Updated
        }
        else {
            DoesNotExist
        }
    }
}


#[test]
fn test_store() {
    let mut kvs = new("/tmp/kvs.json".to_string());
    assert_eq!(kvs.len(), 0);
    assert_eq!(kvs.metrics.last_update, 0);
    assert_eq!(kvs.metrics.size, kvs.len());

    let mut wr: WriteResult;
    let mut lastup: i64;
    wr = kvs.insert("X-Pro2".to_string(), "Fujifilm".to_string());
    assert_eq!(wr, Inserted);
    assert_eq!(kvs.len(), 1);
    assert_ne!(kvs.metrics.last_update, 0);
    assert_eq!(kvs.metrics.size, kvs.len());
    lastup = kvs.metrics.last_update;

    // Make a mistake.
    wr = kvs.insert("D800".to_string(), "Canon".to_string());
    assert_eq!(wr, Inserted);
    assert_eq!(kvs.len(), 2);
    assert_ne!(kvs.metrics.last_update, 0);
    assert!(kvs.metrics.last_update >= lastup);
    assert_eq!(kvs.metrics.size, kvs.len());
    lastup = kvs.metrics.last_update;

    // Fix it.
    wr = kvs.insert("D800".to_string(), "Nikon".to_string());
    assert_eq!(wr, AlreadyExists);
    assert_eq!(kvs.len(), 2);
    assert_ne!(kvs.metrics.last_update, 0);
    assert!(kvs.metrics.last_update >= lastup);
    assert_eq!(kvs.metrics.size, kvs.len());
    lastup = kvs.metrics.last_update;

    wr = kvs.update("D800".to_string(), "Nikon".to_string());
    assert_eq!(wr, Updated);
    assert_eq!(kvs.len(), 2);
    assert_ne!(kvs.metrics.last_update, 0);
    assert!(kvs.metrics.last_update >= lastup);
    assert_eq!(kvs.metrics.size, kvs.len());
    lastup = kvs.metrics.last_update;

    let mut v = kvs.get("D800".to_string());
    assert_eq!(v.expect("missing entry"), "Nikon".to_string());

    v = kvs.get("X-Pro2".to_string());
    assert_eq!(v.expect("missing entry"), "Fujifilm".to_string());
    assert_ne!(kvs.metrics.last_update, 0);
    assert!(kvs.metrics.last_update >= lastup);
    assert_eq!(kvs.metrics.size, kvs.len());
    lastup = kvs.metrics.last_update;

    v = kvs.get("EOS 5D Mark II".to_string());
    assert!(v.is_none());
    assert_ne!(kvs.metrics.last_update, 0);
    assert!(kvs.metrics.last_update >= lastup);
    assert_eq!(kvs.metrics.size, kvs.len());
    lastup = kvs.metrics.last_update;

    wr = kvs.insert("EOS 5D Mark II".to_string(), "Canon".to_string());
    assert_eq!(wr, Inserted);
    assert_ne!(kvs.metrics.last_update, 0);
    assert!(kvs.metrics.last_update >= lastup);
    assert_eq!(kvs.metrics.size, kvs.len());
    assert_eq!(kvs.metrics.size, 3);
    lastup = kvs.metrics.last_update;
    
    // I'd probably not buy a Canon, so...
    wr = kvs.delete("EOS 5D Mark II".to_string());
    assert_eq!(wr, Updated);
    assert_ne!(kvs.metrics.last_update, 0);
    assert!(kvs.metrics.last_update >= lastup);
    assert_eq!(kvs.metrics.size, kvs.len());
    assert_eq!(kvs.metrics.size, 2);
    lastup = kvs.metrics.last_update;

    // just to be certain, NIFO
    wr = kvs.delete("EOS 5D Mark II".to_string());
    assert_eq!(wr, DoesNotExist);
    assert_ne!(kvs.metrics.last_update, 0);
    assert!(kvs.metrics.last_update >= lastup);
    assert_eq!(kvs.metrics.size, kvs.len());
    assert_eq!(kvs.metrics.size, 2);

    kvs.flush().unwrap();
    let kvs2 = Store::load(kvs.path.clone()).unwrap();
    assert_eq!(kvs.metrics.last_write, kvs2.metrics.last_write);
}

