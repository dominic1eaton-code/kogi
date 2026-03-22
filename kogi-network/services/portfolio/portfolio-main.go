/*
 * Kogi Portfolio System frontend service
 * @license
 * @note
 */
package main

import (
	// "kogi.services"
	"encoding/json"
	"fmt"
	"net/http"
)

const (
	serviceID   = "kogi.portfolio"
	serviceName = "kogi-portfolio"
	servicePort = "9002"
)

func HomeHandler(w http.ResponseWriter, r *http.Request) {
	fmt.Fprintf(w, "Welcome to the Home Page!")
}

func CreatePortfolioItem(w http.ResponseWriter, r *http.Request) {
	fmt.Fprintf(w, "create portfolio item")
}

func GetPortfolioItems(w http.ResponseWriter, r *http.Request) {
	fmt.Fprintf(w, "get portfolio item")
}

func ListPortfolioItems(w http.ResponseWriter, r *http.Request) {
	fmt.Fprintf(w, "list portfolio items")
}

func main() {
	// // Initialize a new router
	// r := mux.NewRouter() //

	// // Define a route with a specific path and handler
	// r.HandleFunc("/", HomeHandler).Methods("GET")                               //
	// r.HandleFunc("/portfolio/item", GetPortfolioItems).Methods("GET")           //
	// r.HandleFunc("/portfolio/item/create", CreatePortfolioItem).Methods("POST") //
	// r.HandleFunc("/portfolio/items/list", ListPortfolioItems).Methods("GET")    //

	mux := http.NewServeMux()

	mux.HandleFunc("/api/v2/portfolio/item", func(w http.ResponseWriter, r *http.Request) {
		switch r.Method {
		case http.MethodGet:
			res := "get portfolio item"
			status := http.StatusOK
			payload := res
			w.Header().Set("Content-Type", "application/json")
			w.WriteHeader(status)
			_ = json.NewEncoder(w).Encode(payload)
		case http.MethodPut:
			res := "update portfolio item"
			status := http.StatusOK
			payload := res
			w.Header().Set("Content-Type", "application/json")
			w.WriteHeader(status)
			_ = json.NewEncoder(w).Encode(payload)
		case http.MethodPost:
			res := "create portfolio item"
			status := http.StatusOK
			payload := res
			w.Header().Set("Content-Type", "application/json")
			w.WriteHeader(status)
			_ = json.NewEncoder(w).Encode(payload)
		case http.MethodDelete:
			res := "delete portfolio item"
			status := http.StatusOK
			payload := res
			w.Header().Set("Content-Type", "application/json")
			w.WriteHeader(status)
			_ = json.NewEncoder(w).Encode(payload)
		default:
			res := "get portfolio item"
			status := http.StatusOK
			payload := res
			w.Header().Set("Content-Type", "application/json")
			w.WriteHeader(status)
			_ = json.NewEncoder(w).Encode(payload)
		}
	})

	// Start the server
	// http.ListenAndServe(":8000", r) //
	http.ListenAndServe(":8000", mux) //
}
