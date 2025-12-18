package api

import (
	"context"
	"fmt"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/danielgtaylor/huma/v2"
)

func getLauncherFilename(osName, artifact, launcherName string) (string, error) {
	lowerName := strings.ToLower(strings.ReplaceAll(launcherName, " ", "_"))
	switch osName {
	case "windows":
		if artifact != "exe" {
			return "", fmt.Errorf("invalid artifact for windows: %s", artifact)
		}
		return fmt.Sprintf("%s.exe", launcherName), nil
	case "macos":
		switch artifact {
		case "dmg":
			return fmt.Sprintf("%s.dmg", launcherName), nil
		case "archive":
			// Produced by CI as `${LOWER_LAUNCHER_NAME}_macos.tar.gz`
			return fmt.Sprintf("%s_macos.tar.gz", lowerName), nil
		default:
			return "", fmt.Errorf("invalid artifact for macos: %s", artifact)
		}
	case "linux":
		switch artifact {
		case "bin":
			return lowerName, nil
		case "flatpak":
			return fmt.Sprintf("%s.flatpak", lowerName), nil
		case "flatpakref":
			return fmt.Sprintf("%s.flatpakref", lowerName), nil
		default:
			return "", fmt.Errorf("invalid artifact for linux: %s", artifact)
		}
	default:
		return "", fmt.Errorf("invalid os: %s", osName)
	}
}

func launcherFileMode(osName, artifact string) os.FileMode {
	if osName == "windows" && artifact == "exe" {
		return 0o755
	}
	if osName == "linux" && artifact == "bin" {
		return 0o755
	}
	return 0o644
}

const maxLauncherUploadBytes int64 = 300 * 1024 * 1024

type ArtifactResponse struct {
	ContentDisposition string `header:"Content-Disposition"`
	ContentType        string `header:"Content-Type"`
	Body               []byte `content:"application/octet-stream"`
}

type VersionResponse struct {
	ContentType string `header:"Content-Type"`
	Body        []byte `content:"text/plain"`
}

func registerLaunchers(api huma.API, deps *Dependencies) {
	huma.Register(api, huma.Operation{
		OperationID: "get-launcher-artifact",
		Method:      http.MethodGet,
		Path:        "/launchers/{os}/{artifact}",
		Summary:     "Download launcher artifact",
		Description: "Download launcher artifact for the given OS and artifact type.",
		Tags:        []string{"Launchers"},
	}, func(ctx context.Context, input *struct {
		OS       string `path:"os" enum:"windows,macos,linux" doc:"Operating system"`
		Artifact string `path:"artifact" enum:"exe,dmg,archive,bin,flatpak,flatpakref" doc:"Artifact type"`
	}) (*ArtifactResponse, error) {
		filename, err := getLauncherFilename(input.OS, input.Artifact, deps.Config.LauncherName)
		if err != nil {
			return nil, huma.Error400BadRequest(err.Error())
		}
		dir := filepath.Join(deps.Config.LauncherDir, input.OS, input.Artifact)
		path := filepath.Join(dir, filename)

		raw, err := os.ReadFile(path)
		if err != nil {
			if os.IsNotExist(err) {
				return nil, huma.Error404NotFound("artifact not uploaded")
			}
			return nil, huma.Error500InternalServerError("failed to read artifact")
		}

		return &ArtifactResponse{
			ContentDisposition: fmt.Sprintf("attachment; filename=%q", filename),
			ContentType:        "application/octet-stream",
			Body:               raw,
		}, nil
	})

	huma.Register(api, huma.Operation{
		OperationID: "get-launcher-version",
		Method:      http.MethodGet,
		Path:        "/launchers/{os}/{artifact}/version",
		Summary:     "Get launcher artifact version",
		Description: "Return the version string for the latest uploaded launcher artifact.",
		Tags:        []string{"Launchers"},
	}, func(ctx context.Context, input *struct {
		OS       string `path:"os" enum:"windows,macos,linux" doc:"Operating system"`
		Artifact string `path:"artifact" enum:"exe,dmg,archive,bin,flatpak,flatpakref" doc:"Artifact type"`
	}) (*VersionResponse, error) {
		if _, err := getLauncherFilename(input.OS, input.Artifact, deps.Config.LauncherName); err != nil {
			return nil, huma.Error400BadRequest(err.Error())
		}

		dir := filepath.Join(deps.Config.LauncherDir, input.OS, input.Artifact)
		versionPath := filepath.Join(dir, "version.txt")

		raw, err := os.ReadFile(versionPath)
		if err != nil {
			if os.IsNotExist(err) {
				return nil, huma.Error404NotFound("artifact not uploaded")
			}
			return nil, huma.Error500InternalServerError("failed to read version")
		}

		return &VersionResponse{
			ContentType: "text/plain; charset=utf-8",
			Body:        raw,
		}, nil
	})

	huma.Register(api, huma.Operation{
		OperationID:  "upload-launcher",
		Method:       http.MethodPost,
		Path:         "/launchers/{os}/{artifact}",
		Summary:      "Upload launcher artifact",
		Description:  "Upload launcher artifact for an OS/artifact pair.",
		Tags:         []string{"Launchers"},
		MaxBodyBytes: maxLauncherUploadBytes,
		Security: []map[string][]string{
			{"bearerAuth": {}},
		},
	}, func(ctx context.Context, input *struct {
		AuthHeaders
		OS       string `path:"os" enum:"windows,macos,linux" doc:"Operating system"`
		Artifact string `path:"artifact" enum:"exe,dmg,archive,bin,flatpak,flatpakref" doc:"Artifact type"`
		Version  string `query:"version" doc:"Launcher version identifier (e.g. git sha)"`
		RawBody  []byte
	}) (*struct{}, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}

		version := strings.TrimSpace(input.Version)
		if version == "" {
			return nil, huma.Error400BadRequest("version is required")
		}
		if len(input.RawBody) == 0 {
			return nil, huma.Error400BadRequest("empty upload")
		}

		filename, err := getLauncherFilename(input.OS, input.Artifact, deps.Config.LauncherName)
		if err != nil {
			return nil, huma.Error400BadRequest(err.Error())
		}

		dir := filepath.Join(deps.Config.LauncherDir, input.OS, input.Artifact)
		if err := os.MkdirAll(dir, 0o755); err != nil {
			deps.Logger.Error("failed to create launcher dir", "dir", dir, "error", err)
			return nil, huma.Error500InternalServerError("failed to create directory")
		}

		path := filepath.Join(dir, filename)
		mode := launcherFileMode(input.OS, input.Artifact)
		if err := os.WriteFile(path, input.RawBody, mode); err != nil {
			deps.Logger.Error("failed to write launcher file", "path", path, "error", err)
			return nil, huma.Error500InternalServerError("failed to write file")
		}

		_ = os.WriteFile(filepath.Join(dir, "version.txt"), []byte(version+"\n"), 0o644)

		deps.Logger.Info(
			"launcher uploaded",
			"os", input.OS,
			"artifact", input.Artifact,
			"version", version,
			"filename", filename,
			"mode", mode,
			"ts", time.Now().UTC().Format(time.RFC3339),
		)
		return nil, nil
	})
}
