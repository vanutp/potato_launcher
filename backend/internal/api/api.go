package api

import (
	"log/slog"
	"net/http"
	"strings"

	"github.com/danielgtaylor/huma/v2"
	"github.com/danielgtaylor/huma/v2/adapters/humachi"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/cors"

	"github.com/Petr1Furious/potato-launcher/backend/internal/config"
	"github.com/Petr1Furious/potato-launcher/backend/internal/models"
	"github.com/Petr1Furious/potato-launcher/backend/internal/services"
)

type SpecStore interface {
	GetSpec() (*models.Spec, error)
	Update(func(*models.Spec) error) (*models.Spec, error)
}

type Dependencies struct {
	Config *config.Config
	Store  SpecStore
	Auth   *services.AuthService
	Runner *services.RunnerService
	Hub    *services.Hub
	Logger *slog.Logger
}

func NewAPI(deps *Dependencies) (huma.API, chi.Router) {
	root := chi.NewRouter()
	root.Use(cors.Handler(cors.Options{
		AllowedOrigins: deps.Config.AllowedOrigins,
		AllowedMethods: []string{"GET", "POST", "PATCH", "DELETE", "OPTIONS"},
		AllowedHeaders: []string{"Accept", "Authorization", "Content-Type"},
	}))

	apiRouter := chi.NewRouter()
	root.Mount("/api/v1", apiRouter)

	apiRouter.Get("/ws", deps.Hub.HandleWebSocket)

	cfg := huma.DefaultConfig("Potato Launcher Backend", "1.0.0")
	cfg.OpenAPIPath = "/openapi"
	cfg.DocsPath = ""
	cfg.OpenAPI.OpenAPI = "3.1.0"
	cfg.OpenAPI.Info.Description = "Go rewrite powered by Huma"
	cfg.OpenAPI.Servers = []*huma.Server{{URL: "/api/v1"}}
	cfg.OpenAPI.Components.SecuritySchemes = map[string]*huma.SecurityScheme{
		"bearerAuth": {
			Type:         "http",
			Scheme:       "bearer",
			BearerFormat: "JWT",
		},
	}

	if cfg.OpenAPI.Paths == nil {
		cfg.OpenAPI.Paths = make(map[string]*huma.PathItem)
	}
	cfg.OpenAPI.Paths["/ws"] = &huma.PathItem{
		Get: &huma.Operation{
			OperationID: "connect-websocket",
			Summary:     "Connect WebSocket",
			Description: "Establish a WebSocket connection for real-time updates (build logs, notifications). Expects a 'token' query parameter for authentication.",
			Tags:        []string{"System"},
			Parameters: []*huma.Param{
				{
					Name:        "token",
					In:          "query",
					Description: "Access token",
					Required:    true,
					Schema:      &huma.Schema{Type: "string"},
				},
			},
			Responses: map[string]*huma.Response{
				"101": {Description: "Switching Protocols"},
				"401": {Description: "Unauthorized"},
			},
		},
	}

	api := humachi.New(apiRouter, cfg)

	apiRouter.Get("/docs", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "text/html")
		w.Write([]byte(`<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <meta name="description" content="SwaggerUI" />
  <title>SwaggerUI</title>
  <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5.11.0/swagger-ui.css" />
</head>
<body>
<div id="swagger-ui"></div>
<script src="https://unpkg.com/swagger-ui-dist@5.11.0/swagger-ui-bundle.js" crossorigin></script>
<script>
  window.onload = () => {
    window.ui = SwaggerUIBundle({
      url: '/api/v1/openapi.json',
      dom_id: '#swagger-ui',
    });
  };
</script>
</body>
</html>`))
	})

	registerAuth(api, deps)
	registerSettings(api, deps)
	registerInstances(api, deps)
	registerMCVersions(api, deps)

	return api, root
}

type AuthHeaders struct {
	Authorization string `header:"Authorization" hidden:"true"`
}

func (d *Dependencies) ensureAuth(header string) error {
	if header == "" {
		return huma.Error401Unauthorized("missing Authorization header")
	}
	parts := strings.SplitN(header, " ", 2)
	if len(parts) != 2 || !strings.EqualFold(parts[0], "bearer") {
		return huma.Error401Unauthorized("expected Bearer token")
	}
	if _, err := d.Auth.ValidateToken(parts[1]); err != nil {
		d.Logger.Warn("invalid token attempt", "error", err)
		return huma.Error401Unauthorized("invalid token")
	}
	return nil
}
