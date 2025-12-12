package api

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/danielgtaylor/huma/v2"
)

type UploadLauncherRequest struct {
	Authorization string `header:"Authorization"`
	OS            string `path:"os" enum:"windows,macos,linux" doc:"Operating system"`
	File          []byte `body:"file" content:"application/octet-stream" doc:"Launcher binary file"`
}

func getLauncherFilename(osName, launcherName string) string {
	lowerName := strings.ToLower(strings.ReplaceAll(launcherName, " ", "_"))
	switch osName {
	case "windows":
		return fmt.Sprintf("%s.exe", launcherName)
	case "macos":
		return fmt.Sprintf("%s.dmg", launcherName)
	case "linux":
		return lowerName
	default:
		return ""
	}
}

func registerLaunchers(api huma.API, deps *Dependencies) {
	huma.Register(api, huma.Operation{
		OperationID: "upload-launcher",
		Method:      "POST",
		Path:        "/launchers/{os}",
		Summary:     "Upload launcher binary",
		Description: "Upload a launcher binary for a specific operating system",
		Tags:        []string{"Launchers"},
		Security: []map[string][]string{
			{"bearerAuth": {}},
		},
	}, func(ctx context.Context, req *UploadLauncherRequest) (*struct{}, error) {
		if err := deps.ensureAuth(req.Authorization); err != nil {
			return nil, err
		}

		filename := getLauncherFilename(req.OS, deps.Config.LauncherName)
		if filename == "" {
			return nil, huma.Error400BadRequest("invalid os")
		}

		path := filepath.Join(deps.Config.LauncherDir, filename)
		if err := os.WriteFile(path, req.File, 0o755); err != nil {
			deps.Logger.Error("failed to write launcher file", "path", path, "error", err)
			return nil, huma.Error500InternalServerError("failed to write file")
		}

		return nil, nil
	})
}
