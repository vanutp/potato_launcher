package main

import (
	"log/slog"
	"net/http"
	"os"

	"github.com/Petr1Furious/potato-launcher/backend/internal/api"
	"github.com/Petr1Furious/potato-launcher/backend/internal/config"
	"github.com/Petr1Furious/potato-launcher/backend/internal/services"
	store "github.com/Petr1Furious/potato-launcher/backend/internal/storage"
)

func main() {
	logger := slog.New(slog.NewTextHandler(os.Stdout, nil))
	slog.SetDefault(logger)

	cfg, err := config.Load()
	if err != nil {
		logger.Error("failed to load config", "error", err)
		os.Exit(1)
	}

	store, err := store.New(cfg.SpecFile, cfg.ReplaceDownloadURLs)
	if err != nil {
		logger.Error("failed to init storage", "error", err)
		os.Exit(1)
	}

	deps := &api.Dependencies{
		Config: cfg,
		Store:  store,
		Auth:   services.NewAuthService(cfg),
		Runner: services.NewRunnerService(cfg, store, logger),
		Logger: logger,
	}

	_, router := api.NewAPI(deps)

	logger.Info("starting server", "address", cfg.Address())
	if err := http.ListenAndServe(cfg.Address(), router); err != nil {
		logger.Error("server failed", "error", err)
		os.Exit(1)
	}
}
