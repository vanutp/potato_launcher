#!/bin/sh
set -eu

# Path to the runtime config.js file
CONFIG_FILE=/usr/share/nginx/html/config.js

INDEX_HTML=/usr/share/nginx/html/index.html

TITLE_NAME="${LAUNCHER_NAME:-${VITE_LAUNCHER_NAME:-Potato Launcher}}"

if [ -f "$INDEX_HTML" ]; then
  sed -i "s|<title>.*</title>|<title>${TITLE_NAME}</title>|" "$INDEX_HTML" || true
fi

echo "Generating runtime configuration in $CONFIG_FILE"
cat <<EOF > $CONFIG_FILE
window.config = {
    VITE_GITHUB_URL: "${VITE_GITHUB_URL:-}",
    VITE_LAUNCHER_NAME: "${TITLE_NAME}"
};
if (window.config && window.config.VITE_LAUNCHER_NAME) {
    document.title = window.config.VITE_LAUNCHER_NAME;
}
EOF
