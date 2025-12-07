package api

import (
	"context"
	"errors"
	"fmt"
	"io"
	"mime/multipart"
	"os"
	"path/filepath"
	"strings"
	"time"

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

	cfg := huma.DefaultConfig("Potato Launcher Backend", "2.0.0")
	cfg.OpenAPI.Info.Description = "Go rewrite powered by Huma."
	cfg.OpenAPI.Servers = []*huma.Server{{URL: "/api/v1"}}

	api := humachi.New(apiRouter, cfg)

	registerAuth(api, deps)
	registerSettings(api, deps)
	registerModpacks(api, deps)
	registerMCVersions(api, deps)

	return api, root
}

type AuthHeaders struct {
	Authorization string `header:"Authorization" doc:"Bearer <token>"`
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
		return huma.Error401Unauthorized("invalid token")
	}
	return nil
}

var (
	errVersionExists   = errors.New("modpack already exists")
	errVersionNotFound = errors.New("modpack not found")
)

func registerAuth(api huma.API, deps *Dependencies) {
	type loginInput struct {
		Body models.TokenRequest
	}
	type loginOutput struct {
		Body models.TokenResponse
	}

	huma.Post(api, "/auth/login", func(ctx context.Context, input *loginInput) (*loginOutput, error) {
		if input.Body.Token != deps.Config.AdminSecretToken {
			return nil, huma.Error401Unauthorized("invalid token")
		}
		token, err := deps.Auth.CreateAccessToken("single_user")
		if err != nil {
			return nil, huma.Error500InternalServerError("failed to sign token")
		}
		return &loginOutput{
			Body: models.TokenResponse{
				AccessToken: token,
				TokenType:   "bearer",
			},
		}, nil
	}, huma.OperationTags("Authorization"))
}

func registerSettings(api huma.API, deps *Dependencies) {
	type settingsInput struct {
		AuthHeaders
	}
	type settingsOutput struct {
		Body models.SpecSettings
	}

	huma.Get(api, "/settings", func(ctx context.Context, input *settingsInput) (*settingsOutput, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}
		spec, err := deps.Store.GetSpec()
		if err != nil {
			return nil, huma.Error500InternalServerError(err.Error())
		}
		return &settingsOutput{Body: specToSettings(spec)}, nil
	}, huma.OperationTags("Settings"))

	type updateInput struct {
		AuthHeaders
		Body models.SpecSettings
	}

	huma.Post(api, "/settings", func(ctx context.Context, input *updateInput) (*settingsOutput, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}
		spec, err := deps.Store.Update(func(spec *models.Spec) error {
			applySettings(spec, input.Body)
			return nil
		})
		if err != nil {
			return nil, huma.Error500InternalServerError(err.Error())
		}
		return &settingsOutput{Body: specToSettings(spec)}, nil
	}, huma.OperationTags("Settings"))
}

func registerModpacks(api huma.API, deps *Dependencies) {
	type listInput struct{ AuthHeaders }
	type listOutput struct {
		Body []models.VersionSpec
	}

	huma.Get(api, "/modpacks", func(ctx context.Context, input *listInput) (*listOutput, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}
		spec, err := deps.Store.GetSpec()
		if err != nil {
			return nil, huma.Error500InternalServerError(err.Error())
		}
		return &listOutput{Body: spec.Versions}, nil
	}, huma.OperationTags("Modpacks"))

	type modpackOutput struct {
		Body models.VersionSpec
	}
	type createInput struct {
		AuthHeaders
		Body models.VersionSpec
	}

	huma.Post(api, "/modpacks", func(ctx context.Context, input *createInput) (*modpackOutput, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}
		version := input.Body
		if err := normalizeVersion(deps.Config, &version); err != nil {
			return nil, huma.Error422UnprocessableEntity(err.Error())
		}
		updated, err := deps.Store.Update(func(spec *models.Spec) error {
			if idx := versionIndex(spec, version.Name); idx != -1 {
				return errVersionExists
			}
			spec.Versions = append(spec.Versions, version)
			return nil
		})
		if err != nil {
			return nil, mapVersionError(err)
		}
		_, created := findVersion(updated, version.Name)
		if created == nil {
			return nil, huma.Error500InternalServerError("failed to create modpack")
		}
		return &modpackOutput{Body: *created}, nil
	}, huma.OperationTags("Modpacks"))

	type byNameInput struct {
		AuthHeaders
		Name string `path:"name"`
	}

	huma.Get(api, "/modpacks/{name}", func(ctx context.Context, input *byNameInput) (*modpackOutput, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}
		spec, err := deps.Store.GetSpec()
		if err != nil {
			return nil, huma.Error500InternalServerError(err.Error())
		}
		_, version := findVersion(spec, input.Name)
		if version == nil {
			return nil, huma.Error404NotFound("modpack not found")
		}
		return &modpackOutput{Body: *version}, nil
	}, huma.OperationTags("Modpacks"))

	type updateInput struct {
		AuthHeaders
		Name string `path:"name"`
		Body models.VersionSpec
	}

	huma.Patch(api, "/modpacks/{name}", func(ctx context.Context, input *updateInput) (*modpackOutput, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}
		version := input.Body
		if strings.TrimSpace(version.Name) == "" {
			version.Name = input.Name
		}
		if err := normalizeVersion(deps.Config, &version); err != nil {
			return nil, huma.Error422UnprocessableEntity(err.Error())
		}
		updated, err := deps.Store.Update(func(spec *models.Spec) error {
			idx, _ := findVersion(spec, input.Name)
			if idx == -1 {
				return errVersionNotFound
			}
			if version.Name != input.Name {
				if other := versionIndex(spec, version.Name); other != -1 {
					return errVersionExists
				}
			}
			spec.Versions[idx] = version
			return nil
		})
		if err != nil {
			return nil, mapVersionError(err)
		}
		_, current := findVersion(updated, version.Name)
		return &modpackOutput{Body: *current}, nil
	}, huma.OperationTags("Modpacks"))

	huma.Delete(api, "/modpacks/{name}", func(ctx context.Context, input *byNameInput) (*struct{}, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}
		_, err := deps.Store.Update(func(spec *models.Spec) error {
			idx, _ := findVersion(spec, input.Name)
			if idx == -1 {
				return errVersionNotFound
			}
			spec.Versions = append(spec.Versions[:idx], spec.Versions[idx+1:]...)
			return nil
		})
		if err != nil {
			return nil, mapVersionError(err)
		}
		return &struct{}{}, nil
	}, huma.OperationTags("Modpacks"))

	type buildInput struct{ AuthHeaders }
	type buildOutput struct {
		Body struct {
			Status string `json:"status"`
		}
	}

	huma.Post(api, "/modpacks/build", func(ctx context.Context, input *buildInput) (*buildOutput, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}
		spec, err := deps.Store.GetSpec()
		if err != nil {
			return nil, huma.Error500InternalServerError(err.Error())
		}
		if len(spec.Versions) == 0 {
			return nil, huma.Error400BadRequest("at least one modpack required")
		}
		if err := deps.Runner.RunBuild(ctx); err != nil {
			return nil, huma.Error409Conflict(err.Error())
		}
		return &buildOutput{Body: struct {
			Status string `json:"status"`
		}{Status: "scheduled"}}, nil
	}, huma.OperationTags("Modpacks"))

	type statusOutput struct {
		Body struct {
			Status models.BuildStatus `json:"status"`
		}
	}

	huma.Get(api, "/modpacks/build/status", func(ctx context.Context, input *buildInput) (*statusOutput, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}
		return &statusOutput{Body: struct {
			Status models.BuildStatus `json:"status"`
		}{Status: deps.Runner.Status()}}, nil
	}, huma.OperationTags("Modpacks"))

	registerUpload(api, deps)
}

func registerUpload(api huma.API, deps *Dependencies) {
	type uploadInput struct {
		AuthHeaders
		Name  string                  `path:"name"`
		Files []*multipart.FileHeader `form:"files"`
	}

	huma.Post(api, "/modpacks/{name}/files", func(ctx context.Context, input *uploadInput) (*struct{}, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}
		if len(input.Files) == 0 {
			return nil, huma.Error400BadRequest("no files uploaded")
		}
		spec, err := deps.Store.GetSpec()
		if err != nil {
			return nil, huma.Error500InternalServerError(err.Error())
		}
		_, version := findVersion(spec, input.Name)
		if version == nil {
			return nil, huma.Error404NotFound("modpack not found")
		}
		targetDir, err := resolveIncludeDir(deps.Config, version)
		if err != nil {
			return nil, huma.Error400BadRequest(err.Error())
		}
		if err := saveUploadedFiles(targetDir, input.Files, deps.Config); err != nil {
			return nil, huma.Error500InternalServerError(err.Error())
		}
		return &struct{}{}, nil
	}, huma.OperationTags("Modpacks"))
}

func registerMCVersions(api huma.API, deps *Dependencies) {
	type listInput struct {
		AuthHeaders
	}
	type listOutput struct {
		Body []string
	}

	huma.Get(api, "/mc-versions", func(ctx context.Context, input *listInput) (*listOutput, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}
		versions, err := services.GetVanillaVersions(ctx, "")
		if err != nil {
			return nil, huma.Error503ServiceUnavailable(err.Error())
		}
		return &listOutput{Body: versions}, nil
	}, huma.OperationTags("MC Versions"))

	type loadersInput struct {
		AuthHeaders
		Version string `path:"version"`
	}
	type loadersOutput struct {
		Body []models.LoaderType
	}

	huma.Get(api, "/mc-versions/{version}/loaders", func(ctx context.Context, input *loadersInput) (*loadersOutput, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}
		loaders, err := services.GetLoadersForVersion(ctx, input.Version)
		if err != nil {
			return nil, huma.Error503ServiceUnavailable(err.Error())
		}
		return &loadersOutput{Body: loaders}, nil
	}, huma.OperationTags("MC Versions"))

	type loaderVersionsInput struct {
		AuthHeaders
		Version string            `path:"version"`
		Loader  models.LoaderType `path:"loader"`
	}
	type loaderVersionsOutput struct {
		Body []string
	}

	huma.Get(api, "/mc-versions/{version}/{loader}", func(ctx context.Context, input *loaderVersionsInput) (*loaderVersionsOutput, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}
		versions, err := services.GetLoaderVersions(ctx, input.Version, input.Loader)
		if err != nil {
			return nil, huma.Error503ServiceUnavailable(err.Error())
		}
		return &loaderVersionsOutput{Body: versions}, nil
	}, huma.OperationTags("MC Versions"))
}

func specToSettings(spec *models.Spec) models.SpecSettings {
	return models.SpecSettings{
		ReplaceDownloadURLs: spec.ReplaceDownloadURLs,
	}
}

func applySettings(spec *models.Spec, settings models.SpecSettings) {
	spec.ReplaceDownloadURLs = settings.ReplaceDownloadURLs
}

func versionIndex(spec *models.Spec, name string) int {
	for i := range spec.Versions {
		if spec.Versions[i].Name == name {
			return i
		}
	}
	return -1
}

func findVersion(spec *models.Spec, name string) (int, *models.VersionSpec) {
	for i := range spec.Versions {
		if spec.Versions[i].Name == name {
			return i, &spec.Versions[i]
		}
	}
	return -1, nil
}

func mapVersionError(err error) error {
	switch {
	case errors.Is(err, errVersionExists):
		return huma.Error409Conflict("modpack already exists")
	case errors.Is(err, errVersionNotFound):
		return huma.Error404NotFound("modpack not found")
	default:
		return huma.Error500InternalServerError(err.Error())
	}
}

func ensureIncludeFrom(cfg *config.Config, version *models.VersionSpec) {
	if strings.TrimSpace(version.IncludeFrom) == "" {
		slug := slugifyName(version.Name)
		version.IncludeFrom = filepath.ToSlash(filepath.Join(cfg.UploadedModpacksDir, slug))
	}
}

func ensureAuthBackend(version *models.VersionSpec) {
	if version.AuthBackend == nil {
		version.AuthBackend = &models.AuthBackend{Type: models.AuthOffline}
	}
}

func normalizeVersion(cfg *config.Config, version *models.VersionSpec) error {
	version.Name = strings.TrimSpace(version.Name)
	if version.Name == "" {
		return fmt.Errorf("name is required")
	}
	version.MinecraftVersion = strings.TrimSpace(version.MinecraftVersion)
	if version.MinecraftVersion == "" {
		return fmt.Errorf("minecraft_version is required")
	}
	if version.LoaderName == "" {
		version.LoaderName = models.LoaderVanilla
	}
	if version.LoaderName != models.LoaderVanilla && strings.TrimSpace(version.LoaderVersion) == "" {
		return fmt.Errorf("loader_version is required")
	}

	ensureIncludeFrom(cfg, version)
	ensureAuthBackend(version)
	return nil
}

func slugifyName(name string) string {
	name = strings.TrimSpace(strings.ToLower(name))
	if name == "" {
		return "modpack"
	}
	var builder strings.Builder
	lastDash := false
	for _, r := range name {
		if (r >= 'a' && r <= 'z') || (r >= '0' && r <= '9') {
			builder.WriteRune(r)
			lastDash = false
			continue
		}
		if !lastDash {
			builder.WriteRune('-')
			lastDash = true
		}
	}
	slug := strings.Trim(builder.String(), "-")
	if slug == "" {
		return "modpack"
	}
	return slug
}

func resolveIncludeDir(cfg *config.Config, version *models.VersionSpec) (string, error) {
	includeFrom := strings.TrimSpace(version.IncludeFrom)
	if includeFrom == "" {
		return "", errors.New("modpack include_from is empty")
	}
	if !filepath.IsAbs(includeFrom) {
		includeFrom = filepath.Join(cfg.UploadedModpacksDir, includeFrom)
	}
	abs, err := filepath.Abs(includeFrom)
	if err != nil {
		return "", err
	}
	if !withinBase(cfg.UploadedModpacksDir, abs) {
		return "", fmt.Errorf("include_from must be inside %s", cfg.UploadedModpacksDir)
	}
	return abs, nil
}

func withinBase(base, target string) bool {
	rel, err := filepath.Rel(base, target)
	return err == nil && !strings.HasPrefix(rel, "..")
}

func saveUploadedFiles(targetDir string, files []*multipart.FileHeader, cfg *config.Config) error {
	tmpDir := filepath.Join(cfg.TempDir, fmt.Sprintf("modpack-%d", time.Now().UnixNano()))
	if err := os.RemoveAll(tmpDir); err != nil {
		return err
	}
	if err := os.MkdirAll(tmpDir, 0o755); err != nil {
		return err
	}
	defer os.RemoveAll(tmpDir)

	for i, fileHeader := range files {
		if err := copyUploadedFile(fileHeader, tmpDir, i); err != nil {
			return err
		}
	}

	if err := os.RemoveAll(targetDir); err != nil && !os.IsNotExist(err) {
		return err
	}
	return os.Rename(tmpDir, targetDir)
}

func copyUploadedFile(fileHeader *multipart.FileHeader, dstDir string, fallback int) error {
	src, err := fileHeader.Open()
	if err != nil {
		return err
	}
	defer src.Close()

	rel, err := sanitizeRelative(fileHeader.Filename)
	if err != nil || rel == "" {
		rel = fmt.Sprintf("file_%d", fallback)
	}
	rel = stripTopFolder(rel)
	dstPath := filepath.Join(dstDir, filepath.FromSlash(rel))
	if err := os.MkdirAll(filepath.Dir(dstPath), 0o755); err != nil {
		return err
	}

	dst, err := os.Create(dstPath)
	if err != nil {
		return err
	}
	defer dst.Close()

	if _, err := io.Copy(dst, src); err != nil {
		return err
	}
	return nil
}

func sanitizeRelative(p string) (string, error) {
	p = strings.TrimSpace(p)
	p = strings.TrimLeft(p, "/\\")
	if p == "" {
		return "", fmt.Errorf("empty path")
	}
	clean := filepath.ToSlash(filepath.Clean(p))
	if strings.HasPrefix(clean, "..") || strings.HasPrefix(clean, "/") {
		return "", fmt.Errorf("invalid relative path: %s", p)
	}
	return clean, nil
}

func stripTopFolder(p string) string {
	parts := strings.Split(p, "/")
	if len(parts) <= 1 {
		return p
	}
	return strings.Join(parts[1:], "/")
}
