package api

import "github.com/Petr1Furious/potato-launcher/backend/internal/models"

func toAPISettings(spec *models.Spec) Settings {
	return Settings{
		ReplaceDownloadURLs: spec.ReplaceDownloadURLs,
	}
}

func applySettingsToSpec(spec *models.Spec, settings Settings) {
	spec.ReplaceDownloadURLs = settings.ReplaceDownloadURLs
}

func toAPIInstance(v models.VersionSpec) Instance {
	m := Instance{
		Name:             v.Name,
		MinecraftVersion: v.MinecraftVersion,
		LoaderName:       v.LoaderName,
		LoaderVersion:    v.LoaderVersion,
		RecommendedXmx:   v.RecommendedXmx,
	}

	if v.AuthBackend != nil {
		m.AuthBackend = &AuthBackend{
			Type:         v.AuthBackend.Type,
			AuthBaseURL:  v.AuthBackend.AuthBaseURL,
			ClientID:     v.AuthBackend.ClientID,
			ClientSecret: v.AuthBackend.ClientSecret,
		}
	}

	if len(v.Include) > 0 {
		m.Include = make([]IncludeRule, len(v.Include))
		for i, rule := range v.Include {
			m.Include[i] = IncludeRule{
				Path:        rule.Path,
				Overwrite:   rule.Overwrite,
				Recursive:   rule.Recursive,
				DeleteExtra: rule.DeleteExtra,
			}
		}
	}

	return m
}

func toModelInstance(m Instance) models.VersionSpec {
	v := models.VersionSpec{
		Name:             m.Name,
		MinecraftVersion: m.MinecraftVersion,
		LoaderName:       m.LoaderName,
		LoaderVersion:    m.LoaderVersion,
		RecommendedXmx:   m.RecommendedXmx,
	}

	if m.AuthBackend != nil {
		v.AuthBackend = &models.AuthBackend{
			Type:         m.AuthBackend.Type,
			AuthBaseURL:  m.AuthBackend.AuthBaseURL,
			ClientID:     m.AuthBackend.ClientID,
			ClientSecret: m.AuthBackend.ClientSecret,
		}
	}

	if len(m.Include) > 0 {
		v.Include = make([]models.IncludeRule, len(m.Include))
		for i, rule := range m.Include {
			v.Include[i] = models.IncludeRule{
				Path:        rule.Path,
				Overwrite:   rule.Overwrite,
				Recursive:   rule.Recursive,
				DeleteExtra: rule.DeleteExtra,
			}
		}
	}

	return v
}
