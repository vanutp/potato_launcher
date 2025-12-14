package api

import (
	"context"
	"net/http"

	"github.com/danielgtaylor/huma/v2"

	"github.com/Petr1Furious/potato-launcher/backend/internal/models"
)

func registerSettings(api huma.API, deps *Dependencies) {
	huma.Register(api, huma.Operation{
		OperationID: "get-settings",
		Method:      http.MethodGet,
		Path:        "/settings",
		Summary:     "Get Settings",
		Description: "Retrieve current application settings.",
		Tags:        []string{"Settings"},
		Security:    []map[string][]string{{"bearerAuth": {}}},
	}, func(ctx context.Context, input *struct {
		AuthHeaders
	}) (*struct {
		Body APISettings
	}, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}
		spec, err := deps.Store.GetSpec()
		if err != nil {
			return nil, huma.Error500InternalServerError(err.Error())
		}
		return &struct{ Body APISettings }{Body: toAPISettings(spec)}, nil
	})

	huma.Register(api, huma.Operation{
		OperationID: "update-settings",
		Method:      http.MethodPost,
		Path:        "/settings",
		Summary:     "Update Settings",
		Description: "Update application settings.",
		Tags:        []string{"Settings"},
		Security:    []map[string][]string{{"bearerAuth": {}}},
	}, func(ctx context.Context, input *struct {
		AuthHeaders
		Body APISettings
	}) (*struct {
		Body APISettings
	}, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}
		spec, err := deps.Store.Update(func(spec *models.BuilderSpec) error {
			applySettingsToSpec(spec, input.Body)
			return nil
		})
		if err != nil {
			return nil, huma.Error500InternalServerError(err.Error())
		}
		deps.Logger.Info("settings updated", "replace_download_urls", input.Body.ReplaceDownloadURLs)
		return &struct{ Body APISettings }{Body: toAPISettings(spec)}, nil
	})
}
