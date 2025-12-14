package store

import (
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"sync"

	"github.com/Petr1Furious/potato-launcher/backend/internal/models"
)

type Store struct {
	path string
	mu   sync.RWMutex
}

func New(path string, initial *models.Spec) (*Store, error) {
	if path == "" {
		return nil, errors.New("spec path is required")
	}
	if _, err := os.Stat(path); errors.Is(err, os.ErrNotExist) {
		if initial == nil {
			initial = &models.Spec{}
		}
		if initial.Versions == nil {
			initial.Versions = []models.VersionSpec{}
		}
		if err := writeFile(path, initial); err != nil {
			return nil, err
		}
	}
	return &Store{path: path}, nil
}

func (s *Store) GetSpec() (*models.Spec, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return readFile(s.path)
}

func (s *Store) Update(mutator func(*models.Spec) error) (*models.Spec, error) {
	s.mu.Lock()
	defer s.mu.Unlock()

	spec, err := readFile(s.path)
	if err != nil {
		return nil, err
	}
	if spec.Versions == nil {
		spec.Versions = []models.VersionSpec{}
	}
	if err := mutator(spec); err != nil {
		return nil, err
	}
	if err := writeFile(s.path, spec); err != nil {
		return nil, err
	}
	return spec, nil
}

func readFile(path string) (*models.Spec, error) {
	raw, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("read spec: %w", err)
	}
	if len(raw) == 0 {
		return &models.Spec{Versions: []models.VersionSpec{}}, nil
	}
	var spec models.Spec
	if err := json.Unmarshal(raw, &spec); err != nil {
		return nil, fmt.Errorf("decode spec: %w", err)
	}
	if spec.Versions == nil {
		spec.Versions = []models.VersionSpec{}
	}
	return &spec, nil
}

func writeFile(path string, spec *models.Spec) error {
	if spec.Versions == nil {
		spec.Versions = []models.VersionSpec{}
	}
	raw, err := json.MarshalIndent(spec, "", "  ")
	if err != nil {
		return fmt.Errorf("encode spec: %w", err)
	}
	if err := os.WriteFile(path, raw, 0o644); err != nil {
		return fmt.Errorf("write spec: %w", err)
	}
	return nil
}
