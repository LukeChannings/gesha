package api

type PIDEnabled struct {
	Running bool `json:"running"`
	Heating bool `json:"heating"`
}
