#!/usr/bin/env bash
set -euo pipefail

# Remote instance fetch helper.
# Downloads spec.json and/or selected uploaded instance files from the server to local machine.
#
# Requirements: bash, rsync, ssh

log() { printf >&2 "%s\n" "$*"; }
die() { printf >&2 "error: %s\n" "$*"; exit 1; }

usage() {
  cat <<'EOF'
Usage:
  # Fetch spec.json only
  scripts/remote-instance-fetch.sh --remote user@host --internal-dir /abs/path/to/internal --spec-out ./spec.json

  # Fetch one instance directory
  scripts/remote-instance-fetch.sh --remote user@host --internal-dir /abs/path/to/internal \
    --instance "My Instance=./out/My Instance"

  # Fetch spec + multiple instances
  scripts/remote-instance-fetch.sh --remote user@host --internal-dir /abs/path/to/internal --spec-out ./out/spec.json \
    --instance "A=./out/A" \
    --instance "B=./out/B"

Key options:
  --remote USER@HOST          SSH destination (or set PL_REMOTE)
  --ssh-port PORT             SSH port (default 22; or set PL_SSH_PORT)
  --internal-dir DIR          Remote absolute path to the backend 'internal' directory (or set PL_INTERNAL_DIR)

What to fetch:
  --spec-out PATH             Download <internal-dir>/spec.json and write it to PATH
  --instance "NAME=DIR"       Repeatable; download <internal-dir>/uploaded-instances/NAME/ into DIR/

Behavior:
  --delete                    Pass --delete to rsync to delete local files that are not on the remote (off by default)
  --dry-run                   Print what would happen (and use rsync --dry-run)
EOF
}

REMOTE="${PL_REMOTE:-}"
SSH_PORT="${PL_SSH_PORT:-22}"
INTERNAL_DIR="${PL_INTERNAL_DIR:-}"

SPEC_OUT=""

declare -a INSTANCE_MAPPINGS=()

DO_DELETE=0
DRY_RUN=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help) usage; exit 0 ;;
    --remote) REMOTE="${2:-}"; shift 2 ;;
    --ssh-port) SSH_PORT="${2:-}"; shift 2 ;;
    --internal-dir) INTERNAL_DIR="${2:-}"; shift 2 ;;
    --spec-out) SPEC_OUT="${2:-}"; shift 2 ;;
    --instance) INSTANCE_MAPPINGS+=("${2:-}"); shift 2 ;;
    --delete) DO_DELETE=1; shift ;;
    --dry-run) DRY_RUN=1; shift ;;
    *)
      die "unknown argument: $1 (use --help)"
      ;;
  esac
done

[[ -n "$REMOTE" ]] || die "--remote is required (or set PL_REMOTE)"
[[ -n "$INTERNAL_DIR" ]] || die "--internal-dir is required (or set PL_INTERNAL_DIR)"
[[ "$INTERNAL_DIR" == /* ]] || die "--internal-dir must be an absolute path (got: $INTERNAL_DIR)"

instance_count=${#INSTANCE_MAPPINGS[@]}
if [[ -z "$SPEC_OUT" && "$instance_count" -eq 0 ]]; then
  die "nothing to fetch: use --spec-out PATH and/or one/more --instance \"NAME=DIR\""
fi

if ! command -v rsync >/dev/null 2>&1; then
  die "rsync is required on your local machine"
fi
if ! command -v ssh >/dev/null 2>&1; then
  die "ssh is required on your local machine"
fi

rsync_base=(rsync -az -e "ssh -p $SSH_PORT")
if [[ "$DO_DELETE" -eq 1 ]]; then
  rsync_base+=(--delete)
fi
if [[ "$DRY_RUN" -eq 1 ]]; then
  rsync_base+=(--dry-run)
fi

run_cmd() {
  if [[ "$DRY_RUN" -eq 1 ]]; then
    log "[dry-run] $*"
  fi
  "$@"
}

if [[ -n "$SPEC_OUT" ]]; then
  remote_spec="${INTERNAL_DIR%/}/spec.json"
  mkdir -p "$(dirname "$SPEC_OUT")"
  log "Fetching spec -> ${SPEC_OUT}"
  run_cmd "${rsync_base[@]}" "${REMOTE}:${remote_spec}" "$SPEC_OUT"
fi

for mapping in "${INSTANCE_MAPPINGS[@]+"${INSTANCE_MAPPINGS[@]}"}"; do
  inst="${mapping%%=*}"
  local_dir="${mapping#*=}"
  [[ -n "$inst" ]] || die "bad --instance mapping '$mapping': empty name"
  [[ -n "$local_dir" ]] || die "bad --instance mapping '$mapping': empty local dir"

  remote_instance_dir="${INTERNAL_DIR%/}/uploaded-instances/${inst}/"
  mkdir -p "$local_dir"
  log "Fetching instance '${inst}' -> ${local_dir}/"
  run_cmd "${rsync_base[@]}" "${REMOTE}:${remote_instance_dir}" "${local_dir%/}/"
done

log "Done."
