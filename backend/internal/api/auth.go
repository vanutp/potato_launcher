package api

import (
	"context"
	"net/http"

	"github.com/danielgtaylor/huma/v2"
)

func registerAuth(api huma.API, deps *Dependencies) {
	huma.Register(api, huma.Operation{
		OperationID: "login",
		Method:      http.MethodPost,
		Path:        "/auth/login",
		Summary:     "Login",
		Description: "Exchange an admin token for a session token.",
		Tags:        []string{"Authorization"},
	}, func(ctx context.Context, input *struct {
		Body TokenRequest
	}) (*struct {
		Body TokenResponse
	}, error) {
		if input.Body.Token != deps.Config.AdminSecretToken {
			deps.Logger.Warn("login failed: invalid admin token")
			return nil, huma.Error401Unauthorized("invalid token")
		}
		token, err := deps.Auth.CreateAccessToken("single_user")
		if err != nil {
			deps.Logger.Error("failed to create access token", "error", err)
			return nil, huma.Error500InternalServerError("failed to sign token")
		}
		deps.Logger.Info("login successful")
		return &struct {
			Body TokenResponse
		}{
			Body: TokenResponse{
				AccessToken: token,
				TokenType:   "bearer",
			},
		}, nil
	})
}

func registerAuthCheck(api huma.API, deps *Dependencies) {
	huma.Register(api, huma.Operation{
		OperationID: "auth-check",
		Method:      http.MethodGet,
		Path:        "/auth/check",
		Summary:     "Auth check",
		Description: "Check if the provided Authorization header contains a valid admin JWT. Returns 204 when authorized.",
		Tags:        []string{"System"},
		Security:    []map[string][]string{{"bearerAuth": {}}},
		Responses: map[string]*huma.Response{
			"204": {Description: "Authorized"},
			"401": {Description: "Unauthorized"},
		},
	}, func(ctx context.Context, input *struct {
		AuthHeaders
	}) (*struct{}, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}
		return nil, nil
	})
}
