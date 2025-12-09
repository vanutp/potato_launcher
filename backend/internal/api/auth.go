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
			return nil, huma.Error401Unauthorized("invalid token")
		}
		token, err := deps.Auth.CreateAccessToken("single_user")
		if err != nil {
			return nil, huma.Error500InternalServerError("failed to sign token")
		}
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
