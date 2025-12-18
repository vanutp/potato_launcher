# Server setup (Docker Compose)

The backend + filebrowser + nginx (with frontend) can be deployed with one `docker-compose.yml`.

::: info Requirements
- A Linux server with a public IP address
- A domain name
- Docker + Docker Compose plugin installed
:::

## Quickstart

1. Download `docker-compose.yml` and `.env.example` from the repository.

```bash
mkdir potato-launcher-backend
cd potato-launcher-backend
wget https://raw.githubusercontent.com/Petr1Furious/potato-launcher/refs/heads/master/docker-compose.yml
wget -O .env https://raw.githubusercontent.com/Petr1Furious/potato-launcher/refs/heads/master/.env.example
wget https://raw.githubusercontent.com/Petr1Furious/potato-launcher/refs/heads/master/nginx.conf
```

1. Edit `.env` and set at least:

- **`ADMIN_JWT_SECRET`**: JWT signing secret (generate with `openssl rand -base64 48`)
- **`ADMIN_SECRET_TOKEN`**: admin “password” used to log in (pick a long random password)
- **`DOWNLOAD_SERVER_BASE`**: public URL where your server will host generated instance files (usually `https://<your-domain>/data`)

3. Start the setup:

```bash
docker compose up -d
```

By default, nginx is exposed on **port 8000** (see `docker-compose.yml`). You can change this to any other value and put it behind any reverse proxy.

## What gets served where

When the setup is up:

- **Web UI**: `http://<host>:8000/`
- **Backend API**: `http://<host>:8000/api/v1/`
- **API docs**: `http://<host>:8000/api/v1/docs`
- **Generated instances & metadata**: `http://<host>:8000/data/`
- **Admin panel** `http://<host>:8000/admin/` (requires login)

## Logging in

Open the Web UI and log in using your `ADMIN_SECRET_TOKEN`.

## Persistent data / backups

All persistent state lives under `./state/` by default:

- `state/metadata`: instance spec + settings
- `state/uploaded-instances`: modpack file source (use Filebrowser to manage)
- `state/generated`: generated output served under `/data`
- `state/launcher`: uploaded launcher artifacts
- `state/filebrowser-db`: Filebrowser database

Back up the whole `state/` directory.

::: warning
Do not share `ADMIN_SECRET_TOKEN` or `ADMIN_JWT_SECRET`. Anyone with the admin token can log in and upload/modify instances and launchers.
:::
