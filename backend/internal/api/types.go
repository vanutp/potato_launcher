package api

import "github.com/Petr1Furious/potato-launcher/backend/internal/models"

// Auth

type TokenRequest struct {
	Token string `json:"token" doc:"Admin secret token"`
}

type TokenResponse struct {
	AccessToken string `json:"access_token"`
	TokenType   string `json:"token_type"`
}

// Settings

type Settings struct {
	ReplaceDownloadURLs bool `json:"replace_download_urls" doc:"Whether to replace download URLs in the client"`
}

// Instances

type AuthBackend struct {
	Type         models.AuthType `json:"type"`
	AuthBaseURL  string          `json:"auth_base_url,omitempty"`
	ClientID     string          `json:"client_id,omitempty"`
	ClientSecret string          `json:"client_secret,omitempty"`
}

type IncludeRule struct {
	Path        string `json:"path"`
	Overwrite   *bool  `json:"overwrite,omitempty"`
	Recursive   *bool  `json:"recursive,omitempty"`
	DeleteExtra *bool  `json:"delete_extra,omitempty"`
}

type Instance struct {
	Name             string            `json:"name" example:"survival-1.21"`
	MinecraftVersion string            `json:"minecraft_version" example:"1.21.1"`
	LoaderName       models.LoaderType `json:"loader_name" example:"fabric"`
	LoaderVersion    string            `json:"loader_version,omitempty" example:"0.15.11"`
	Include          []IncludeRule     `json:"include,omitempty"`
	AuthBackend      *AuthBackend      `json:"auth_backend,omitempty"`
	RecommendedXmx   string            `json:"recommended_xmx,omitempty" example:"4G"`
}

type BuildStatusResponse struct {
	Status models.BuildStatus `json:"status"`
}
