package api

import (
	"os"
	"path/filepath"
	"strings"

	"github.com/Petr1Furious/potato-launcher/backend/internal/config"
	"github.com/Petr1Furious/potato-launcher/backend/internal/models"
)

func toAPISettings(spec *models.BuilderSpec) APISettings {
	return APISettings{
		ReplaceDownloadURLs: spec.ReplaceDownloadURLs,
	}
}

func applySettingsToSpec(spec *models.BuilderSpec, settings APISettings) {
	spec.ReplaceDownloadURLs = settings.ReplaceDownloadURLs
}

func toAPIInstance(v models.BuilderInstance) APIInstance {
	return APIInstance{
		Name:             v.Name,
		MinecraftVersion: v.MinecraftVersion,
		LoaderName:       v.LoaderName,
		LoaderVersion:    v.LoaderVersion,
		RecommendedXmx:   v.RecommendedXmx,
		Include:          v.Include,
		AuthBackend:      v.AuthBackend,
	}
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

func getInstanceDir(cfg *config.Config, instanceName string) string {
	return filepath.Join(cfg.UploadedInstancesDir, slugifyName(instanceName))
}

func ensureIncludeFrom(cfg *config.Config, instance *models.BuilderInstance) {
	instance.IncludeFrom = filepath.ToSlash(getInstanceDir(cfg, instance.Name))
}

func ensureInstanceDir(cfg *config.Config, instanceName string) error {
	dir := getInstanceDir(cfg, instanceName)
	return os.MkdirAll(dir, 0o755)
}

func ensureAuthBackend(instance *models.BuilderInstance) {
	if instance.AuthBackend == nil {
		instance.AuthBackend = &models.AuthBackend{Type: models.AuthOffline}
	}
}

func normalizeInstance(cfg *config.Config, instance *models.BuilderInstance) error {
	instance.Name = strings.TrimSpace(instance.Name)
	if instance.Name == "" {
		return NewValidationError("name", "name is required")
	}
	instance.MinecraftVersion = strings.TrimSpace(instance.MinecraftVersion)
	if instance.MinecraftVersion == "" {
		return NewValidationError("minecraft_version", "minecraft_version is required")
	}
	if instance.LoaderName == "" {
		instance.LoaderName = models.LoaderVanilla
	}
	if instance.LoaderName != models.LoaderVanilla && strings.TrimSpace(instance.LoaderVersion) == "" {
		return NewValidationError("loader_version", "loader_version is required")
	}

	ensureIncludeFrom(cfg, instance)
	ensureAuthBackend(instance)
	return nil
}

func toBuilderInstance(cfg *config.Config, m APIInstance) (*models.BuilderInstance, error) {
	instance := models.BuilderInstance{
		Name:             m.Name,
		MinecraftVersion: m.MinecraftVersion,
		LoaderName:       m.LoaderName,
		LoaderVersion:    m.LoaderVersion,
		RecommendedXmx:   m.RecommendedXmx,
		Include:          m.Include,
		AuthBackend:      m.AuthBackend,
	}
	if err := normalizeInstance(cfg, &instance); err != nil {
		return nil, err
	}
	return &instance, nil
}
