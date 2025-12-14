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

func New(path string, initial *models.BuilderSpec) (*Store, error) {
	if path == "" {
		return nil, errors.New("spec path is required")
	}
	if _, err := os.Stat(path); errors.Is(err, os.ErrNotExist) {
		if initial == nil {
			initial = &models.BuilderSpec{}
		}
		if initial.Instances == nil {
			initial.Instances = []models.BuilderInstance{}
		}
		if err := writeFile(path, initial); err != nil {
			return nil, err
		}
	}
	return &Store{path: path}, nil
}

func (s *Store) GetSpec() (*models.BuilderSpec, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return readFile(s.path)
}

func (s *Store) Update(mutator func(*models.BuilderSpec) error) (*models.BuilderSpec, error) {
	s.mu.Lock()
	defer s.mu.Unlock()

	spec, err := readFile(s.path)
	if err != nil {
		return nil, err
	}
	if spec.Instances == nil {
		spec.Instances = []models.BuilderInstance{}
	}
	if err := mutator(spec); err != nil {
		return nil, err
	}
	if err := writeFile(s.path, spec); err != nil {
		return nil, err
	}
	return spec, nil
}

func readFile(path string) (*models.BuilderSpec, error) {
	raw, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("read spec: %w", err)
	}
	if len(raw) == 0 {
		return &models.BuilderSpec{Instances: []models.BuilderInstance{}}, nil
	}
	var spec models.BuilderSpec
	if err := json.Unmarshal(raw, &spec); err != nil {
		return nil, fmt.Errorf("decode spec: %w", err)
	}
	if spec.Instances == nil {
		spec.Instances = []models.BuilderInstance{}
	}
	return &spec, nil
}

func writeFile(path string, spec *models.BuilderSpec) error {
	if spec.Instances == nil {
		spec.Instances = []models.BuilderInstance{}
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
