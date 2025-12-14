package api

import (
	"context"
	"errors"
	"net/http"

	"github.com/danielgtaylor/huma/v2"

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
		Body []APIInstance
	}, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}
		spec, err := deps.Store.GetSpec()
		if err != nil {
			return nil, huma.Error500InternalServerError(err.Error())
		}

		instances := make([]APIInstance, len(spec.Instances))
		for i, v := range spec.Instances {
			instances[i] = toAPIInstance(v)
		}
		return &struct{ Body []APIInstance }{Body: instances}, nil
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
		Body APIInstance
	}) (*struct {
		Body APIInstance
	}, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}

		instance, err := toBuilderInstance(deps.Config, input.Body)
		if err != nil {
			return nil, mapAppError(err)
		}

		updated, err := deps.Store.Update(func(spec *models.BuilderSpec) error {
			if idx := instanceIndex(spec, instance.Name); idx != -1 {
				return errInstanceExists
			}
			spec.Instances = append(spec.Instances, *instance)
			return nil
		})
		if err != nil {
			return nil, mapAppError(err)
		}

		_, created := findInstance(updated, instance.Name)
		if created == nil {
			return nil, huma.Error500InternalServerError("failed to create instance")
		}

		if err := ensureInstanceDir(deps.Config, instance.Name); err != nil {
			deps.Logger.Warn("failed to create instance directory", "name", instance.Name, "error", err)
		}

		deps.Logger.Info("instance created", "name", instance.Name)
		return &struct{ Body APIInstance }{Body: toAPIInstance(*created)}, nil
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
		Body APIInstance
	}, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}
		spec, err := deps.Store.GetSpec()
		if err != nil {
			return nil, huma.Error500InternalServerError(err.Error())
		}
		_, instance := findInstance(spec, input.Name)
		if instance == nil {
			return nil, huma.Error404NotFound("instance not found")
		}
		return &struct{ Body APIInstance }{Body: toAPIInstance(*instance)}, nil
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
		Body APIInstance
	}) (*struct {
		Body APIInstance
	}, error) {
		if err := deps.ensureAuth(input.Authorization); err != nil {
			return nil, err
		}

		newInstance, err := toBuilderInstance(deps.Config, input.Body)
		if err != nil {
			return nil, mapAppError(err)
		}

		updated, err := deps.Store.Update(func(spec *models.BuilderSpec) error {
			idx := instanceIndex(spec, input.Name)
			if idx == -1 {
				return errInstanceNotFound
			}

			if newInstance.Name != input.Name {
				if other := instanceIndex(spec, newInstance.Name); other != -1 {
					return errInstanceExists
				}
			}

			spec.Instances[idx] = *newInstance
			return nil
		})

		if err != nil {
			return nil, mapAppError(err)
		}

		_, current := findInstance(updated, newInstance.Name)
		deps.Logger.Info("instance updated", "name", input.Name, "new_name", newInstance.Name)
		return &struct{ Body APIInstance }{Body: toAPIInstance(*current)}, nil
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
		_, err := deps.Store.Update(func(spec *models.BuilderSpec) error {
			idx := instanceIndex(spec, input.Name)
			if idx == -1 {
				return errInstanceNotFound
			}
			spec.Instances = append(spec.Instances[:idx], spec.Instances[idx+1:]...)
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
		if len(spec.Instances) == 0 {
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
}

func instanceIndex(spec *models.BuilderSpec, name string) int {
	for i := range spec.Instances {
		if spec.Instances[i].Name == name {
			return i
		}
	}
	return -1
}

func findInstance(spec *models.BuilderSpec, name string) (int, *models.BuilderInstance) {
	for i := range spec.Instances {
		if spec.Instances[i].Name == name {
			return i, &spec.Instances[i]
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
