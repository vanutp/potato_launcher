package models

import "time"

type Spec struct {
	ReplaceDownloadURLs bool          `json:"replace_download_urls"`
	Versions            []VersionSpec `json:"versions"`
}

type SpecSettings struct {
	ReplaceDownloadURLs bool `json:"replace_download_urls"`
}

type BuilderSpec struct {
	DownloadServerBase  string        `json:"download_server_base"`
	ResourcesURLBase    *string       `json:"resources_url_base,omitempty"`
	ReplaceDownloadURLs bool          `json:"replace_download_urls"`
	ExecBeforeAll       string        `json:"exec_before_all,omitempty"`
	ExecAfterAll        string        `json:"exec_after_all,omitempty"`
	Versions            []VersionSpec `json:"versions"`
}

type LoaderType string

const (
	LoaderVanilla LoaderType = "vanilla"
	LoaderForge   LoaderType = "forge"
	LoaderFabric  LoaderType = "fabric"
	LoaderNeo     LoaderType = "neoforge"
)

type AuthType string

const (
	AuthMojang   AuthType = "mojang"
	AuthTelegram AuthType = "telegram"
	AuthEly      AuthType = "ely.by"
	AuthOffline  AuthType = "offline"
)

type AuthBackend struct {
	Type         AuthType `json:"type"`
	AuthBaseURL  string   `json:"auth_base_url,omitempty"`
	ClientID     string   `json:"client_id,omitempty"`
	ClientSecret string   `json:"client_secret,omitempty"`
}

type IncludeRule struct {
	Path        string `json:"path"`
	Overwrite   *bool  `json:"overwrite,omitempty"`
	Recursive   *bool  `json:"recursive,omitempty"`
	DeleteExtra *bool  `json:"delete_extra,omitempty"`
}

type VersionSpec struct {
	Name             string        `json:"name"`
	MinecraftVersion string        `json:"minecraft_version"`
	LoaderName       LoaderType    `json:"loader_name"`
	LoaderVersion    string        `json:"loader_version,omitempty"`
	IncludeFrom      string        `json:"include_from,omitempty"`
	Include          []IncludeRule `json:"include,omitempty"`
	AuthBackend      *AuthBackend  `json:"auth_backend,omitempty"`
	RecommendedXmx   string        `json:"recommended_xmx,omitempty"`
	ExecBefore       string        `json:"exec_before,omitempty"`
	ExecAfter        string        `json:"exec_after,omitempty"`
}

type TokenRequest struct {
	Token string `json:"token"`
}

type TokenResponse struct {
	AccessToken string `json:"access_token"`
	TokenType   string `json:"token_type"`
}

type BuildStatus string

const (
	BuildRunning BuildStatus = "running"
	BuildIdle    BuildStatus = "idle"
)

type JWTClaims struct {
	Sub string `json:"sub"`
	exp time.Time
}
