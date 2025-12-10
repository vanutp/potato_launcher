package main

import (
	"log"
	"net/http"

	"github.com/Petr1Furious/potato-launcher/backend/internal/api"
	"github.com/Petr1Furious/potato-launcher/backend/internal/config"
	"github.com/Petr1Furious/potato-launcher/backend/internal/services"
	store "github.com/Petr1Furious/potato-launcher/backend/internal/storage"
)

func main() {
	cfg, err := config.Load()
	if err != nil {
		log.Fatalf("load config: %v", err)
	}

	store, err := store.New(cfg.SpecFile, cfg.ReplaceDownloadURLs)
	if err != nil {
		log.Fatalf("init storage: %v", err)
	}

	deps := &api.Dependencies{
		Config: cfg,
		Store:  store,
		Auth:   services.NewAuthService(cfg),
		Runner: services.NewRunnerService(cfg, store),
	}

	_, router := api.NewAPI(deps)

	log.Printf("listening on %s", cfg.Address())
	if err := http.ListenAndServe(cfg.Address(), router); err != nil {
		log.Fatalf("server error: %v", err)
	}
}
