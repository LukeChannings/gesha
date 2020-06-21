package api

type PIDEnabled struct {
	Enabled bool `json:"enabled"`
	Heating bool `json:"heating"`
}
