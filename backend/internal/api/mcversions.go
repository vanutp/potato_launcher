package api

import (
	"context"
	"net/http"

	"github.com/danielgtaylor/huma/v2"

	"github.com/Petr1Furious/potato-launcher/backend/internal/models"
	"github.com/Petr1Furious/potato-launcher/backend/internal/services"
)

func registerMCVersions(api huma.API, deps *Dependencies) {
	huma.Register(api, huma.Operation{
		OperationID: "list-mc-versions",
		Method:      http.MethodGet,
		Path:        "/mc-versions",
		Summary:     "List Minecraft Versions",
		Description: "Get a list of available Minecraft versions.",
		Tags:        []string{"MC Versions"},
		Security:    []map[string][]string{{"bearerAuth": {}}},
	}, func(ctx context.Context, input *struct {
		AuthHeaders
	}) (*struct {
		Body []string
	}, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}
		versions, err := services.GetVanillaVersions(ctx, "")
		if err != nil {
			return nil, huma.Error503ServiceUnavailable(err.Error())
		}
		return &struct{ Body []string }{Body: versions}, nil
	})

	huma.Register(api, huma.Operation{
		OperationID: "list-loaders",
		Method:      http.MethodGet,
		Path:        "/mc-versions/{version}/loaders",
		Summary:     "List Loaders",
		Description: "Get available loaders for a specific Minecraft version.",
		Tags:        []string{"MC Versions"},
		Security:    []map[string][]string{{"bearerAuth": {}}},
	}, func(ctx context.Context, input *struct {
		AuthHeaders
		Version string `path:"version" doc:"Minecraft version"`
	}) (*struct {
		Body []models.LoaderType
	}, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}
		loaders, err := services.GetLoadersForVersion(ctx, input.Version)
		if err != nil {
			return nil, huma.Error503ServiceUnavailable(err.Error())
		}
		return &struct{ Body []models.LoaderType }{Body: loaders}, nil
	})

	huma.Register(api, huma.Operation{
		OperationID: "list-loader-versions",
		Method:      http.MethodGet,
		Path:        "/mc-versions/{version}/{loader}",
		Summary:     "List Loader Versions",
		Description: "Get specific versions for a loader on a Minecraft version.",
		Tags:        []string{"MC Versions"},
		Security:    []map[string][]string{{"bearerAuth": {}}},
	}, func(ctx context.Context, input *struct {
		AuthHeaders
		Version string            `path:"version" doc:"Minecraft version"`
		Loader  models.LoaderType `path:"loader" doc:"Loader type (e.g. forge, fabric)"`
	}) (*struct {
		Body []string
	}, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}
		versions, err := services.GetLoaderVersions(ctx, input.Version, input.Loader)
		if err != nil {
			return nil, huma.Error503ServiceUnavailable(err.Error())
		}
		return &struct{ Body []string }{Body: versions}, nil
	})
}
