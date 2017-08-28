// kvdemo is a simple key-value store with an HTTP/JSON UI.
//
// To add a key to the store, POST a request to /<keyname> with a
// JSON body containing {'value': <value>}. To retrieve a key, send
// a GET request to /<keyname>. GETting the root will return some
// metrics for the server.
//
// The store is persisted to disk as a JSON file.
package main

import (
	"bytes"
	"encoding/json"
	"flag"
	"fmt"
	"io/ioutil"
	"log"
	"net/http"
	"os"
)

// A Response contains the HTTP status code and result of an endpoint. It
// is exported so that it may be serialised by the JSON package.
type Response struct {
	Status int         `json:"status"`
	Data   interface{} `json:"data"`
}

// uploadKey reads value for key from the HTTP request body, updates
// the value in the store, and writes the store to disk. If there is
// an error getting the value (e.g. invalid JSON or no 'value' key in
// the JSON), an HTTP Bad Request is returned. If the store file could
// not be written, an HTTP Internal Server Error is returned.
func uploadKey(w http.ResponseWriter, req *http.Request, key string) *Response {
	var m = map[string]string{}
	in, err := ioutil.ReadAll(req.Body)
	if err != nil {
		return &Response{
			Status: http.StatusBadRequest,
			Data:   err.Error(),
		}
	}

	err = json.Unmarshal(in, &m)
	if err != nil {
		return &Response{
			Status: http.StatusBadRequest,
			Data:   err.Error(),
		}
	}

	value, ok := m["value"]
	if !ok {
		return &Response{
			Status: http.StatusBadRequest,
			Data:   "no value provided for key " + key,
		}
	}

	if setValue(key, value) {
		err = writeStore()
		if err != nil {
			return &Response{
				Status: http.StatusInternalServerError,
				Data:   "server encountered an error storing the key / value pairs",
			}
		}
	}

	return &Response{
		Status: http.StatusOK,
		Data:   "",
	}
}

// retrieveKey looks up key in the store. If it's present, the value is
// returned. Otherwise, an HTTP 404 is returned.
func retrieveKey(w http.ResponseWriter, key string) *Response {
	value, ok := getValue(key)
	if !ok {
		return &Response{
			Status: http.StatusNotFound,
			Data:   fmt.Sprintf("key '%s' doesn't exist in the store", key),
		}
	}

	return &Response{
		Status: http.StatusOK,
		Data:   value,
	}
}

// handler determines which key is being requested. If it's the empty key,
// then the request is for the index. Otherwise, it's a request for an
// operation on a key.
//
// The metrics endpoint only accepts GET requests. Any other method
// results in an HTTP Method Not Allowed error.
//
// If a request for an operation on a key is a GET request, the
// retrieveKey handler is called on the key. If it's a POST request,
// the uploadKey handler is called. Any other method results in an
// HTTP Method Not Allowed Error.
func handler(w http.ResponseWriter, req *http.Request) {
	var r *Response
	key := req.URL.Path[1:]

	if key == "" {
		if req.Method != "GET" {
			r = &Response{
				Data:   "invalid method " + req.Method,
				Status: http.StatusMethodNotAllowed,
			}
		} else {
			r = &Response{
				Status: http.StatusOK,
				Data:   store.metrics,
			}
		}
	} else {
		switch req.Method {
		case "POST":
			r = uploadKey(w, req, key)
		case "GET":
			r = retrieveKey(w, key)
		default:
			r = &Response{
				Data:   "invalid method " + req.Method,
				Status: http.StatusMethodNotAllowed,
			}
		}
	}
	req.Body.Close()

	out, err := json.Marshal(r)
	if err != nil {
		w.WriteHeader(http.StatusInternalServerError)
		w.Write([]byte("error forming response"))
	} else {
		buf := &bytes.Buffer{}
		json.Indent(buf, out, "", "        ")
		w.WriteHeader(r.Status)
		w.Write(buf.Bytes())
	}
}

func main() {
	var addr string

	flag.StringVar(&addr, "a", "localhost:8000", "`address` to listen on")
	flag.StringVar(&store.file, "f", "store.json", "`path` to store data file")
	flag.Parse()

	in, err := ioutil.ReadFile(store.file)
	if err != nil {
		if !os.IsNotExist(err) {
			log.Fatal(err)
		}
	} else {
		err = json.Unmarshal(in, &store.values)
		if err != nil {
			log.Fatal(err)
		}
	}

	setupMetrics()

	http.HandleFunc("/", handler)
	log.Println("listening on", addr)
	log.Fatal(http.ListenAndServe(addr, nil))
}
