package api

import (
	"context"
	"errors"
	"fmt"
	"io"
	"mime/multipart"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/danielgtaylor/huma/v2"

	"github.com/Petr1Furious/potato-launcher/backend/internal/config"
	"github.com/Petr1Furious/potato-launcher/backend/internal/models"
)

var (
	errInstanceExists   = NewConflictError("instance already exists")
	errInstanceNotFound = NewNotFoundError("instance not found")
)

func registerInstances(api huma.API, deps *Dependencies) {
	huma.Register(api, huma.Operation{
		OperationID: "list-instances",
		Method:      http.MethodGet,
		Path:        "/instances",
		Summary:     "List Instances",
		Description: "Get a list of all configured instances.",
		Tags:        []string{"Instances"},
		Security:    []map[string][]string{{"bearerAuth": {}}},
		Responses: map[string]*huma.Response{
			"200": {Description: "List of instances"},
			"500": {Description: "Internal server error"},
		},
	}, func(ctx context.Context, input *struct {
		AuthHeaders
	}) (*struct {
		Body []Instance
	}, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}
		spec, err := deps.Store.GetSpec()
		if err != nil {
			return nil, huma.Error500InternalServerError(err.Error())
		}

		instances := make([]Instance, len(spec.Versions))
		for i, v := range spec.Versions {
			instances[i] = toAPIInstance(v)
		}
		return &struct{ Body []Instance }{Body: instances}, nil
	})

	huma.Register(api, huma.Operation{
		OperationID: "create-instance",
		Method:      http.MethodPost,
		Path:        "/instances",
		Summary:     "Create Instance",
		Description: "Create a new instance configuration.",
		Tags:        []string{"Instances"},
		Security:    []map[string][]string{{"bearerAuth": {}}},
		Responses: map[string]*huma.Response{
			"200": {Description: "Instance created successfully"},
			"409": {Description: "Instance already exists"},
			"422": {Description: "Validation error"},
			"500": {Description: "Internal server error"},
		},
	}, func(ctx context.Context, input *struct {
		AuthHeaders
		Body Instance
	}) (*struct {
		Body Instance
	}, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}

		version := toModelInstance(input.Body)
		if err := normalizeVersion(deps.Config, &version); err != nil {
			return nil, mapAppError(err)
		}

		updated, err := deps.Store.Update(func(spec *models.Spec) error {
			if idx := versionIndex(spec, version.Name); idx != -1 {
				return errInstanceExists
			}
			spec.Versions = append(spec.Versions, version)
			return nil
		})
		if err != nil {
			return nil, mapAppError(err)
		}

		_, created := findVersion(updated, version.Name)
		if created == nil {
			return nil, huma.Error500InternalServerError("failed to create instance")
		}
		deps.Logger.Info("instance created", "name", version.Name)
		return &struct{ Body Instance }{Body: toAPIInstance(*created)}, nil
	})

	huma.Register(api, huma.Operation{
		OperationID: "get-instance",
		Method:      http.MethodGet,
		Path:        "/instances/{name}",
		Summary:     "Get Instance",
		Description: "Get a specific instance by name.",
		Tags:        []string{"Instances"},
		Security:    []map[string][]string{{"bearerAuth": {}}},
		Responses: map[string]*huma.Response{
			"200": {Description: "Instance details"},
			"404": {Description: "Instance not found"},
			"500": {Description: "Internal server error"},
		},
	}, func(ctx context.Context, input *struct {
		AuthHeaders
		Name string `path:"name" doc:"Instance name"`
	}) (*struct {
		Body Instance
	}, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}
		spec, err := deps.Store.GetSpec()
		if err != nil {
			return nil, huma.Error500InternalServerError(err.Error())
		}
		_, version := findVersion(spec, input.Name)
		if version == nil {
			return nil, huma.Error404NotFound("instance not found")
		}
		return &struct{ Body Instance }{Body: toAPIInstance(*version)}, nil
	})

	huma.Register(api, huma.Operation{
		OperationID: "update-instance",
		Method:      http.MethodPatch,
		Path:        "/instances/{name}",
		Summary:     "Update Instance",
		Description: "Update an existing instance configuration.",
		Tags:        []string{"Instances"},
		Security:    []map[string][]string{{"bearerAuth": {}}},
		Responses: map[string]*huma.Response{
			"200": {Description: "Instance updated successfully"},
			"404": {Description: "Instance not found"},
			"409": {Description: "Instance name conflict"},
			"422": {Description: "Validation error"},
			"500": {Description: "Internal server error"},
		},
	}, func(ctx context.Context, input *struct {
		AuthHeaders
		Name string `path:"name" doc:"Instance name"`
		Body Instance
	}) (*struct {
		Body Instance
	}, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}

		newVersion := toModelInstance(input.Body)
		if strings.TrimSpace(newVersion.Name) == "" {
			newVersion.Name = input.Name
		}

		updated, err := deps.Store.Update(func(spec *models.Spec) error {
			idx, existing := findVersion(spec, input.Name)
			if idx == -1 {
				return errInstanceNotFound
			}

			if newVersion.Name != input.Name {
				if other := versionIndex(spec, newVersion.Name); other != -1 {
					return errInstanceExists
				}
			}

			newVersion.ExecBefore = existing.ExecBefore
			newVersion.ExecAfter = existing.ExecAfter
			if newVersion.IncludeFrom == "" {
				newVersion.IncludeFrom = existing.IncludeFrom
			}

			if err := normalizeVersion(deps.Config, &newVersion); err != nil {
				return err
			}

			spec.Versions[idx] = newVersion
			return nil
		})

		if err != nil {
			return nil, mapAppError(err)
		}

		_, current := findVersion(updated, newVersion.Name)
		deps.Logger.Info("instance updated", "name", input.Name, "new_name", newVersion.Name)
		return &struct{ Body Instance }{Body: toAPIInstance(*current)}, nil
	})

	huma.Register(api, huma.Operation{
		OperationID: "delete-instance",
		Method:      http.MethodDelete,
		Path:        "/instances/{name}",
		Summary:     "Delete Instance",
		Description: "Delete an instance configuration.",
		Tags:        []string{"Instances"},
		Security:    []map[string][]string{{"bearerAuth": {}}},
		Responses: map[string]*huma.Response{
			"200": {Description: "Instance deleted successfully"},
			"404": {Description: "Instance not found"},
			"500": {Description: "Internal server error"},
		},
	}, func(ctx context.Context, input *struct {
		AuthHeaders
		Name string `path:"name" doc:"Instance name"`
	}) (*struct{}, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}
		_, err := deps.Store.Update(func(spec *models.Spec) error {
			idx, _ := findVersion(spec, input.Name)
			if idx == -1 {
				return errInstanceNotFound
			}
			spec.Versions = append(spec.Versions[:idx], spec.Versions[idx+1:]...)
			return nil
		})
		if err != nil {
			return nil, mapAppError(err)
		}
		deps.Logger.Info("instance deleted", "name", input.Name)
		return &struct{}{}, nil
	})

	huma.Register(api, huma.Operation{
		OperationID: "build-instances",
		Method:      http.MethodPost,
		Path:        "/instances/build",
		Summary:     "Build Instances",
		Description: "Trigger a build process for all instances.",
		Tags:        []string{"Instances"},
		Security:    []map[string][]string{{"bearerAuth": {}}},
		Responses: map[string]*huma.Response{
			"200": {Description: "Build started successfully"},
			"400": {Description: "No instances to build"},
			"409": {Description: "Build already running"},
			"500": {Description: "Internal server error"},
		},
	}, func(ctx context.Context, input *struct {
		AuthHeaders
	}) (*struct {
		Body struct {
			Status string `json:"status"`
		}
	}, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}
		spec, err := deps.Store.GetSpec()
		if err != nil {
			return nil, huma.Error500InternalServerError(err.Error())
		}
		if len(spec.Versions) == 0 {
			return nil, huma.Error400BadRequest("at least one instance required")
		}
		if err := deps.Runner.RunBuild(ctx); err != nil {
			return nil, huma.Error409Conflict(err.Error())
		}
		deps.Logger.Info("build triggered")
		return &struct {
			Body struct {
				Status string `json:"status"`
			}
		}{Body: struct {
			Status string `json:"status"`
		}{Status: "scheduled"}}, nil
	})

	huma.Register(api, huma.Operation{
		OperationID: "get-build-status",
		Method:      http.MethodGet,
		Path:        "/instances/build/status",
		Summary:     "Get Build Status",
		Description: "Get the current status of the build process.",
		Tags:        []string{"Instances"},
		Security:    []map[string][]string{{"bearerAuth": {}}},
		Responses: map[string]*huma.Response{
			"200": {Description: "Current build status"},
			"401": {Description: "Unauthorized"},
		},
	}, func(ctx context.Context, input *struct {
		AuthHeaders
	}) (*struct {
		Body BuildStatusResponse
	}, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}
		return &struct {
			Body BuildStatusResponse
		}{Body: BuildStatusResponse{Status: deps.Runner.Status()}}, nil
	})

	registerUpload(api, deps)
}

func registerUpload(api huma.API, deps *Dependencies) {
	huma.Register(api, huma.Operation{
		OperationID: "upload-instance-files",
		Method:      http.MethodPost,
		Path:        "/instances/{name}/files",
		Summary:     "Upload Instance Files",
		Description: "Upload files for a specific instance.",
		Tags:        []string{"Instances"},
		Security:    []map[string][]string{{"bearerAuth": {}}},
		Responses: map[string]*huma.Response{
			"200": {Description: "Files uploaded successfully"},
			"400": {Description: "Invalid request (no files or invalid path)"},
			"404": {Description: "Instance not found"},
			"500": {Description: "Internal server error"},
		},
	}, func(ctx context.Context, input *struct {
		AuthHeaders
		Name  string                  `path:"name" doc:"Instance name"`
		Files []*multipart.FileHeader `form:"files" doc:"Files to upload"`
	}) (*struct{}, error) {
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
			return nil, huma.Error404NotFound("instance not found")
		}
		targetDir, err := resolveIncludeDir(deps.Config, version)
		if err != nil {
			return nil, huma.Error400BadRequest(err.Error())
		}
		if err := saveUploadedFiles(targetDir, input.Files, deps.Config); err != nil {
			return nil, huma.Error500InternalServerError(err.Error())
		}
		deps.Logger.Info("files uploaded", "instance", input.Name, "count", len(input.Files))
		return &struct{}{}, nil
	})
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

func mapAppError(err error) error {
	var appErr *AppError
	if errors.As(err, &appErr) {
		switch appErr.Code {
		case ErrCodeConflict:
			return huma.Error409Conflict(appErr.Message)
		case ErrCodeNotFound:
			return huma.Error404NotFound(appErr.Message)
		case ErrCodeValidation:
			return huma.Error422UnprocessableEntity(appErr.Message)
		}
	}
	return huma.Error500InternalServerError(err.Error())
}

func ensureIncludeFrom(cfg *config.Config, version *models.VersionSpec) {
	if strings.TrimSpace(version.IncludeFrom) == "" {
		slug := slugifyName(version.Name)
		version.IncludeFrom = filepath.ToSlash(filepath.Join(cfg.UploadedInstancesDir, slug))
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
		return NewValidationError("name", "name is required")
	}
	version.MinecraftVersion = strings.TrimSpace(version.MinecraftVersion)
	if version.MinecraftVersion == "" {
		return NewValidationError("minecraft_version", "minecraft_version is required")
	}
	if version.LoaderName == "" {
		version.LoaderName = models.LoaderVanilla
	}
	if version.LoaderName != models.LoaderVanilla && strings.TrimSpace(version.LoaderVersion) == "" {
		return NewValidationError("loader_version", "loader_version is required")
	}

	ensureIncludeFrom(cfg, version)
	ensureAuthBackend(version)
	return nil
}

func slugifyName(name string) string {
	name = strings.TrimSpace(strings.ToLower(name))
	if name == "" {
		return "instance"
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
		return "instance"
	}
	return slug
}

func resolveIncludeDir(cfg *config.Config, version *models.VersionSpec) (string, error) {
	includeFrom := strings.TrimSpace(version.IncludeFrom)
	if includeFrom == "" {
		return "", NewValidationError("include_from", "instance include_from is empty")
	}
	if !filepath.IsAbs(includeFrom) {
		includeFrom = filepath.Join(cfg.UploadedInstancesDir, includeFrom)
	}
	abs, err := filepath.Abs(includeFrom)
	if err != nil {
		return "", err
	}
	if !withinBase(cfg.UploadedInstancesDir, abs) {
		return "", NewValidationError("include_from", fmt.Sprintf("include_from must be inside %s", cfg.UploadedInstancesDir))
	}
	return abs, nil
}

func withinBase(base, target string) bool {
	rel, err := filepath.Rel(base, target)
	return err == nil && !strings.HasPrefix(rel, "..")
}

func saveUploadedFiles(targetDir string, files []*multipart.FileHeader, cfg *config.Config) error {
	tmpDir := filepath.Join(cfg.TempDir, fmt.Sprintf("instance-%d", time.Now().UnixNano()))
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
