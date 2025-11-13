SUDO := sudo

.PHONY: build_frontend build_backend up all

build_frontend:
	$(SUDO) docker build -f frontend.dockerfile -t my_frontend .

build_backend:
	$(SUDO) docker build -f backend.dockerfile -t my_backend .

# Сборка обоих образов
all: build_backend build_frontend

# Поднять сервисы из docker-compose.yml
up:
	$(SUDO) docker compose up

down:
	$(SUDO) docker compose down

enter_backend:
	$(SUDO) docker exec -it potato_launcher-backend-1 /bin/bash