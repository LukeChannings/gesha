package api

// OperationResult - Indicates if the operation succeeded, and if not, what the error was.
type OperationResult struct {
	Ok bool `json:"ok"`

	ErrorMessage string `json:"errorMessage,omitempty"`
}
