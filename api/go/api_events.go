/*
 * Domain Tracker
 *
 * This is the domain tracker API that provides details about PingEvents made to various domains.
 *
 * API version: 1.0.0
 * Contact: itsme@aalekhpatel.com
 * Generated by: Swagger Codegen (https://github.com/swagger-api/swagger-codegen.git)
 */

package swagger

import (
	"fmt"
	"net/http"
)

func FindEventsByDomain(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json; charset=UTF-8")
	w.WriteHeader(http.StatusOK)
	fmt.Fprintf(w, "Heheh")
}

func FindEventsbyTimeRange(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json; charset=UTF-8")
	w.WriteHeader(http.StatusOK)
}
