package main

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
)

// Definir la estructura del JSON
type Data struct {
	Faculty    string `json:"faculty"`
	Student    string `json:"student"`
	Age        string `json:"age"`
	Discipline string `json:"discipline"`
}

func dataHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method == http.MethodPost {
		var d Data
		// Decodificar el JSON del cuerpo de la solicitud
		err := json.NewDecoder(r.Body).Decode(&d)
		if err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		// Mostrar los datos
		fmt.Fprintf(w, "Faculty: %s, Student: %s, Age: %s, Discipline: %s\n", d.Faculty, d.Student, d.Age, d.Discipline)
	} else {
		http.Error(w, "Invalid request method", http.StatusMethodNotAllowed)
	}
}

func main() {
	http.HandleFunc("/data", dataHandler)
	log.Println("Listening on :8080")
	log.Fatal(http.ListenAndServe(":8080", nil))
}
