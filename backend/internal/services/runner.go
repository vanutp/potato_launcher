package services

import (
	"context"
	"encoding/json"
	"fmt"
	"log/slog"
	"os"
	"os/exec"
	"sync"

	"github.com/Petr1Furious/potato-launcher/backend/internal/config"
	"github.com/Petr1Furious/potato-launcher/backend/internal/models"
)

type SpecProvider interface {
	GetSpec() (*models.Spec, error)
}

type RunnerService struct {
	cfg     *config.Config
	store   SpecProvider
	status  models.BuildStatus
	mu      sync.RWMutex
	running bool
	logger  *slog.Logger
}

func NewRunnerService(cfg *config.Config, store SpecProvider, logger *slog.Logger) *RunnerService {
	return &RunnerService{
		cfg:    cfg,
		store:  store,
		status: models.BuildIdle,
		logger: logger,
	}
}

func (r *RunnerService) Status() models.BuildStatus {
	r.mu.RLock()
	defer r.mu.RUnlock()
	return r.status
}

func (r *RunnerService) RunBuild(ctx context.Context) error {
	r.mu.Lock()
	if r.running {
		r.mu.Unlock()
		return fmt.Errorf("build already running")
	}
	r.running = true
	r.status = models.BuildRunning
	r.mu.Unlock()

	go r.execute(context.Background())
	return nil
}

func (r *RunnerService) execute(ctx context.Context) {
	r.logger.Info("starting build process")
	if err := r.prepareSpecFile(); err != nil {
		r.finish(err)
		return
	}

	cmd := exec.CommandContext(
		ctx,
		r.cfg.InstanceBuilderBinary,
		"-s",
		r.cfg.SpecFile,
		r.cfg.GeneratedDir,
		r.cfg.WorkdirDir,
	)

	err := cmd.Run()
	r.finish(err)
}

func (r *RunnerService) finish(runErr error) {
	r.mu.Lock()
	r.running = false
	r.status = models.BuildIdle
	r.mu.Unlock()

	if runErr != nil {
		r.logger.Error("runner failed", "error", runErr)
	} else {
		r.logger.Info("build finished successfully")
	}
}

func (r *RunnerService) prepareSpecFile() error {
	spec, err := r.store.GetSpec()
	if err != nil {
		return err
	}
	builderSpec := models.BuilderSpec{
		DownloadServerBase:  r.cfg.DownloadServerBase,
		ResourcesURLBase:    r.cfg.ResourcesURLBase,
		ReplaceDownloadURLs: spec.ReplaceDownloadURLs,
		ExecBeforeAll:       r.cfg.ExecBeforeAll,
		ExecAfterAll:        r.cfg.ExecAfterAll,
		Versions:            spec.Versions,
	}
	raw, err := json.MarshalIndent(builderSpec, "", "    ")
	if err != nil {
		return err
	}
	return os.WriteFile(r.cfg.SpecFile, raw, 0o644)
}
