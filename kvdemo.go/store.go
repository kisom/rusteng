package main

import (
	"encoding/json"
	"io/ioutil"
	"os"
	"sync"
	"time"
)

// Value contains some value contained in the KV store. This is exported
// so that it may be used with the JSON library.
type Value struct {
	Updated int64  // Unix timestamp of last update.
	Version int    // Incremented on each write.
	Value   string // The actual value.
}

// update determines whether the new value is different from the current
// value. If it is, the timestamp is updated, the version is bumped, and
// the value is replaced. The method returns true if the value was replaced
// and false if it wasn't.
func (v *Value) update(s string) bool {
	if s != v.Value {
		v.Updated = time.Now().Unix()
		v.Version++
		v.Value = s
		return true
	}
	return false
}

// Metrics contains basic health check information about the server. This
// is exported so that it may be serialised by the JSON package.
type Metrics struct {
	// Size of key store.
	Size int `json:"size"`

	// Last time the store was successfully written.
	LastWrite int64 `json:"last_write"`

	// Last time a key was changed.
	LastUpdate int64 `json:"last_update"`

	// If a write error has occurred, it will be presented here.
	WriteError string `json:"write_error"`
}

// store is the global data structure containing the data store.
var store = struct {
	// lock is used to prevent concurrent writes.
	lock sync.Mutex

	// values contains the actual key/value pairs.
	values map[string]*Value

	// file contains the path to the store file.
	file string

	// metrics tracks information about the store.
	metrics Metrics
}{
	// values is initialised to an empty map; this is because an
	// attempt to unmarshal JSON into a nil map will panic.
	values: map[string]*Value{},
}

// setupMetrics populates the store's metrics field. This has to be
// done after the store file is loaded, and therefore can't be done
// in an init() function.
//
// The last updated time field in the metrics is set to the latest update
// time across all the values in the key store. The last write time is
// set to the modified time on the store file, and if any error occurs
// trying to read the file (apart from ENOENT), it will go in the last
// write error field.
func setupMetrics() {
	store.metrics.Size = len(store.values)

	for _, v := range store.values {
		if v.Updated > store.metrics.LastUpdate {
			store.metrics.LastUpdate = v.Updated
		}
	}

	fi, err := os.Stat(store.file)
	if err != nil {
		if !os.IsNotExist(err) {
			store.metrics.WriteError = err.Error()
		}
	} else {
		store.metrics.LastWrite = fi.ModTime().Unix()
	}
}

// setValue updates a value in the store and updates the metrics as
// needed. It returns true if the value was changed, and false otherwise.
func setValue(key, value string) bool {
	store.lock.Lock()
	defer store.lock.Unlock()

	v := store.values[key]
	if v == nil {
		v = &Value{}
	}

	if v.update(value) {
		store.values[key] = v
		store.metrics.LastUpdate = time.Now().Unix()
		store.metrics.Size = len(store.values)
		return true
	}

	return false
}

// writeStore flushes the in-memory key/value pairs to disk. It updates
// the metrics as appropriate, including any write errors.
func writeStore() error {
	out, err := json.Marshal(store.values)
	if err != nil {
		store.metrics.WriteError = err.Error()
		return err
	}

	err = ioutil.WriteFile(store.file, out, 0644)
	if err != nil {
		store.metrics.WriteError = err.Error()
		return err
	}

	store.metrics.LastWrite = time.Now().Unix()
	store.metrics.WriteError = ""
	return nil
}

// getValue looks up the key in the store, returning the value if it's
// present. It mimics the same operation on Go's maps.
func getValue(key string) (Value, bool) {
	store.lock.Lock()
	defer store.lock.Unlock()

	v, ok := store.values[key]
	if ok {
		return *v, ok
	}

	return Value{}, false
}
