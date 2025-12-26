#!/usr/bin/env bash
set -euo pipefail

# Remote instance builder helper.
# rsyncs spec.json and raw modpack files to the server and runs instance_builder
#
# Requirements: bash, rsync, ssh

log() { printf >&2 "%s\n" "$*"; }
die() { printf >&2 "error: %s\n" "$*"; exit 1; }

usage() {
  cat <<'EOF'
Usage:
  scripts/remote-instance-build.sh --remote user@host --internal-dir /abs/path/to/internal --spec ./spec.json \
    --instance "Instance A=./packs/a" \
    --instance "Instance B=./packs/b"

Key options:
  --remote USER@HOST          SSH destination (or set PL_REMOTE)
  --ssh-port PORT             SSH port (default 22; or PL_SSH_PORT)
  --internal-dir DIR          Remote absolute path to the backend 'internal' directory (or set PL_INTERNAL_DIR)
  --container NAME            Container name for backend (default potato-launcher-backend; or PL_CONTAINER)
  --spec PATH                 Local spec.json path (or PL_SPEC)
  --docker-host VALUE         Remote DOCKER_HOST for rootless docker (or set PL_DOCKER_HOST). Example: unix:///run/user/1000/docker.sock

Container paths (optional; can also be set via env vars):
  --container-spec PATH       Spec path inside container (default /data/internal/spec.json; or PL_CONTAINER_SPEC)
  --container-generated DIR   Generated dir inside container (default /data/generated; or PL_CONTAINER_GENERATED)
  --container-workdir DIR     Workdir dir inside container (default /data/workdir; or PL_CONTAINER_WORKDIR)

Modpacks:
  --instance "NAME=DIR"        Repeatable; sync DIR for instance NAME

Execution:
  --dry-run                   Print what would happen without changing remote
  --no-build                  Only sync (no docker exec)

Notes:
  - This script does NOT rewrite include_from, you have to set it yourself (typically "/data/internal/uploaded-instances/<instance-name>").
  - Spec is synced to:
      <internal-dir>/spec.json
    and modpacks are synced to:
      <internal-dir>/uploaded-instances/<instance-name>/
EOF
}

REMOTE="${PL_REMOTE:-}"
SSH_PORT="${PL_SSH_PORT:-22}"
INTERNAL_DIR="${PL_INTERNAL_DIR:-}"
CONTAINER="${PL_CONTAINER:-potato-launcher-backend}"
SPEC="${PL_SPEC:-}"
DOCKER_HOST_REMOTE="${PL_DOCKER_HOST:-}"

declare -a INSTANCE_MAPPINGS=() # entries: "Instance Name=/abs/or/rel/path"

CONTAINER_SPEC="${PL_CONTAINER_SPEC:-/data/internal/spec.json}"
CONTAINER_GENERATED="${PL_CONTAINER_GENERATED:-/data/generated}"
CONTAINER_WORKDIR="${PL_CONTAINER_WORKDIR:-/data/workdir}"

DRY_RUN=0
DO_BUILD=1

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help) usage; exit 0 ;;
    --remote) REMOTE="${2:-}"; shift 2 ;;
    --ssh-port) SSH_PORT="${2:-}"; shift 2 ;;
    --internal-dir) INTERNAL_DIR="${2:-}"; shift 2 ;;
    --container) CONTAINER="${2:-}"; shift 2 ;;
    --spec) SPEC="${2:-}"; shift 2 ;;
    --docker-host) DOCKER_HOST_REMOTE="${2:-}"; shift 2 ;;
    --container-spec) CONTAINER_SPEC="${2:-}"; shift 2 ;;
    --container-generated) CONTAINER_GENERATED="${2:-}"; shift 2 ;;
    --container-workdir) CONTAINER_WORKDIR="${2:-}"; shift 2 ;;
    --instance) INSTANCE_MAPPINGS+=("${2:-}"); shift 2 ;;
    --dry-run) DRY_RUN=1; shift ;;
    --no-build) DO_BUILD=0; shift ;;
    *)
      die "unknown argument: $1 (use --help)"
      ;;
  esac
done

[[ -n "$REMOTE" ]] || die "--remote is required (or set PL_REMOTE)"
[[ -n "$INTERNAL_DIR" ]] || die "--internal-dir is required (or set PL_INTERNAL_DIR)"
[[ "$INTERNAL_DIR" == /* ]] || die "--internal-dir must be an absolute path (got: $INTERNAL_DIR)"
[[ -n "$SPEC" ]] || die "--spec is required (or set PL_SPEC)"
[[ -f "$SPEC" ]] || die "spec file not found: $SPEC"

if [[ ${#INSTANCE_MAPPINGS[@]} -eq 0 ]]; then
  die "no modpacks specified (use one/more --modpack \"NAME=DIR\")"
fi

if ! command -v rsync >/dev/null 2>&1; then
  die "rsync is required on your local machine"
fi
if ! command -v ssh >/dev/null 2>&1; then
  die "ssh is required on your local machine"
fi

ssh_base=(ssh -p "$SSH_PORT")
rsync_base=(rsync -az --delete -e "ssh -p $SSH_PORT")
if [[ "$DRY_RUN" -eq 1 ]]; then
  rsync_base+=(--dry-run)
fi

run_cmd() {
  if [[ "$DRY_RUN" -eq 1 ]]; then
    log "[dry-run] $*"
    return 0
  fi
  "$@"
}

quote_remote_path() {
  # Return a single-quoted string safe for remote shell parsing, usable in rsync dest "host:<quoted>".
  # This preserves spaces and most special characters.
  local s="$1"
  s="${s//\'/\'\\\'\'}"
  printf "'%s'" "$s"
}

REMOTE_SPEC_HOST="${INTERNAL_DIR%/}/spec.json"
log "Syncing spec -> ${REMOTE}:${REMOTE_SPEC_HOST}"
run_cmd "${rsync_base[@]}" "$SPEC" "${REMOTE}:$(quote_remote_path "$REMOTE_SPEC_HOST")"

# sync each instance to <internal-dir>/uploaded-instances/<instance-name>/
for mapping in "${INSTANCE_MAPPINGS[@]}"; do
  inst="${mapping%%=*}"
  dir="${mapping#*=}"
  [[ -d "$dir" ]] || die "instance dir not found for '$inst': $dir"

  remote_instance_host="${INTERNAL_DIR%/}/uploaded-instances/${inst}/"
  log "Syncing instance '$inst' ($dir) -> ${REMOTE}:${remote_instance_host}"
  # trailing slash to sync contents into the directory
  run_cmd "${rsync_base[@]}" "${dir%/}/" "${REMOTE}:$(quote_remote_path "$remote_instance_host")"
done

if [[ "$DO_BUILD" -eq 1 ]]; then
  log "Triggering remote build via docker exec in container: $CONTAINER"
  remote_exec=( "${ssh_base[@]}" "$REMOTE" )
  remote_cmd=$(
    cat <<EOF
set -euo pipefail
${DOCKER_HOST_REMOTE:+export DOCKER_HOST=$(printf %q "$DOCKER_HOST_REMOTE")}
docker exec ${CONTAINER} instance_builder -s ${CONTAINER_SPEC} ${CONTAINER_GENERATED} ${CONTAINER_WORKDIR}
EOF
  )

  if [[ "$DRY_RUN" -eq 1 ]]; then
    log "[dry-run] ssh -p $SSH_PORT $REMOTE <<'REMOTE'\n$remote_cmd\nREMOTE"
  else
    "${remote_exec[@]}" "bash -lc $(printf '%q' "$remote_cmd")"
  fi
else
  log "Skipping build (--no-build)."
fi

log "Done."
