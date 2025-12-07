package services

import (
	"context"
	"encoding/json"
	"encoding/xml"
	"errors"
	"fmt"
	"io"
	"net/http"
	"slices"
	"strings"
	"time"

	"github.com/Petr1Furious/potato-launcher/backend/internal/models"
)

const (
	mojangManifestURL   = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json"
	fabricMetaBaseURL   = "https://meta.fabricmc.net/v2/versions/loader/"
	forgeMetadataURL    = "https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json"
	neoforgeMetadataURL = "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml"
)

var httpClient = &http.Client{
	Timeout: 10 * time.Second,
}

func GetVanillaVersions(ctx context.Context, versionType string) ([]string, error) {
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, mojangManifestURL, nil)
	if err != nil {
		return nil, err
	}
	resp, err := httpClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("mojang manifest error: %s", resp.Status)
	}
	var payload struct {
		Versions []struct {
			ID   string `json:"id"`
			Type string `json:"type"`
		} `json:"versions"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&payload); err != nil {
		return nil, err
	}
	out := make([]string, 0, len(payload.Versions))
	for _, v := range payload.Versions {
		if versionType == "" || strings.EqualFold(v.Type, versionType) {
			out = append(out, v.ID)
		}
	}
	return out, nil
}

func GetLoadersForVersion(ctx context.Context, version string) ([]models.LoaderType, error) {
	vanilla, err := GetVanillaVersions(ctx, "")
	if err != nil {
		return nil, err
	}
	loaders := make([]models.LoaderType, 0, 4)
	if slices.Contains(vanilla, version) {
		loaders = append(loaders, models.LoaderVanilla)
	}
	if ok, _ := fabricHasLoader(ctx, version); ok {
		loaders = append(loaders, models.LoaderFabric)
	}
	if ok, _ := forgeHasLoader(ctx, version); ok {
		loaders = append(loaders, models.LoaderForge)
	}
	if ok, _ := neoforgeHasLoader(ctx, version); ok {
		loaders = append(loaders, models.LoaderNeo)
	}
	return loaders, nil
}

func GetLoaderVersions(ctx context.Context, version string, loader models.LoaderType) ([]string, error) {
	switch loader {
	case models.LoaderVanilla:
		return []string{version}, nil
	case models.LoaderFabric:
		return getFabricLoaderVersions(ctx, version)
	case models.LoaderForge:
		return getForgeLoaderVersions(ctx, version)
	case models.LoaderNeo:
		return getNeoforgeLoaderVersions(ctx, version)
	default:
		return nil, errors.New("unknown loader")
	}
}

func fabricHasLoader(ctx context.Context, version string) (bool, error) {
	versions, err := getFabricLoaderVersions(ctx, version)
	return len(versions) > 0, err
}

func getFabricLoaderVersions(ctx context.Context, version string) ([]string, error) {
	url := fabricMetaBaseURL + version
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	if err != nil {
		return nil, err
	}
	resp, err := httpClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()
	if resp.StatusCode == http.StatusNotFound || resp.StatusCode == http.StatusBadRequest {
		return []string{}, nil
	}
	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("fabric meta error: %s", resp.Status)
	}
	var payload []struct {
		Loader struct {
			Version string `json:"version"`
		} `json:"loader"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&payload); err != nil {
		return nil, err
	}
	seen := map[string]struct{}{}
	out := make([]string, 0, len(payload))
	for _, item := range payload {
		if item.Loader.Version == "" {
			continue
		}
		if _, ok := seen[item.Loader.Version]; !ok {
			seen[item.Loader.Version] = struct{}{}
			out = append(out, item.Loader.Version)
		}
	}
	return out, nil
}

func forgeHasLoader(ctx context.Context, version string) (bool, error) {
	data, err := fetchForgeMetadata(ctx)
	if err != nil {
		return false, err
	}
	_, ok := data[version]
	return ok, nil
}

func getForgeLoaderVersions(ctx context.Context, version string) ([]string, error) {
	data, err := fetchForgeMetadata(ctx)
	if err != nil {
		return nil, err
	}
	items := data[version]
	out := make([]string, 0, len(items))
	prefix := version + "-"
	for _, item := range items {
		if strings.HasPrefix(item, prefix) {
			out = append(out, strings.TrimPrefix(item, prefix))
		} else {
			out = append(out, item)
		}
	}
	slices.SortFunc(out, func(a, b string) int {
		if a == b {
			return 0
		}
		return strings.Compare(b, a)
	})
	return out, nil
}

func fetchForgeMetadata(ctx context.Context) (map[string][]string, error) {
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, forgeMetadataURL, nil)
	if err != nil {
		return nil, err
	}
	resp, err := httpClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("forge metadata error: %s", resp.Status)
	}
	var payload map[string][]string
	if err := json.NewDecoder(resp.Body).Decode(&payload); err != nil {
		return nil, err
	}
	return payload, nil
}

func neoforgeHasLoader(ctx context.Context, version string) (bool, error) {
	versions, err := getNeoforgeLoaderVersions(ctx, version)
	return len(versions) > 0, err
}

func getNeoforgeLoaderVersions(ctx context.Context, version string) ([]string, error) {
	prefix := mcToNeoforgePrefix(version)
	if prefix == "" {
		return nil, nil
	}
	items, err := fetchNeoforgeVersions(ctx)
	if err != nil {
		return nil, err
	}
	matched := make([]string, 0)
	for _, item := range items {
		if strings.HasPrefix(item, prefix) {
			matched = append(matched, item)
		}
	}
	return matched, nil
}

func fetchNeoforgeVersions(ctx context.Context) ([]string, error) {
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, neoforgeMetadataURL, nil)
	if err != nil {
		return nil, err
	}
	resp, err := httpClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("neoforge metadata error: %s", resp.Status)
	}
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}
	var payload struct {
		Versioning struct {
			Versions struct {
				Version []string `xml:"version"`
			} `xml:"versions"`
		} `xml:"versioning"`
	}
	if err := xml.Unmarshal(body, &payload); err != nil {
		return nil, err
	}
	return payload.Versioning.Versions.Version, nil
}

func mcToNeoforgePrefix(version string) string {
	parts := strings.Split(version, ".")
	if len(parts) >= 3 {
		return fmt.Sprintf("%s.%s.", parts[1], parts[2])
	}
	if len(parts) == 2 {
		return fmt.Sprintf("%s.0.", parts[1])
	}
	return ""
}
