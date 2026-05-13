#!/bin/sh
# Install pace-coach from a feature branch CI artifact.
# Requires the GitHub CLI (gh) with auth: https://cli.github.com
#
# Usage:
#   sh install-feature-branch.sh <branch>
#   sh install-feature-branch.sh feat/menubar-ux
set -e

REPO="job-almekinders/pace-coach"
INSTALL_DIR="/usr/local/bin"
BRANCH="$1"

if [ -z "$BRANCH" ]; then
    echo "Usage: sh install-feature-branch.sh <branch>" >&2
    exit 1
fi

echo "==> Checking system requirements..."

if [ "$(uname)" != "Darwin" ]; then
    echo "Error: pace-coach only supports macOS." >&2
    exit 1
fi
echo "    OS: macOS $(sw_vers -productVersion)"

if [ "$(uname -m)" != "arm64" ]; then
    echo "Error: pace-coach only supports Apple Silicon (arm64)." >&2
    exit 1
fi
echo "    Arch: arm64 (Apple Silicon)"

if ! command -v gh >/dev/null 2>&1; then
    echo "Error: the GitHub CLI (gh) is required." >&2
    echo "Install it from https://cli.github.com, then run: gh auth login" >&2
    exit 1
fi

echo "==> Finding latest successful CI run for branch '${BRANCH}'..."
RUN_ID=$(gh run list \
    --repo "$REPO" \
    --branch "$BRANCH" \
    --workflow CI \
    --status success \
    --limit 1 \
    --json databaseId \
    --jq '.[0].databaseId')

if [ -z "$RUN_ID" ] || [ "$RUN_ID" = "null" ]; then
    echo "Error: no successful CI run found for branch '$BRANCH'." >&2
    echo "Check https://github.com/${REPO}/actions for run status." >&2
    exit 1
fi
echo "    Run ID: ${RUN_ID}"

TMP=$(mktemp -d)

echo "==> Downloading binaries from CI run ${RUN_ID}..."
gh run download "$RUN_ID" \
    --repo "$REPO" \
    --name pace-coach-binaries \
    --dir "$TMP"

echo "==> Installing to ${INSTALL_DIR} (may require sudo)..."
sudo install -m 755 "${TMP}/pace-coach" "${INSTALL_DIR}/pace-coach"
echo "    Installed: ${INSTALL_DIR}/pace-coach"
sudo install -m 755 "${TMP}/pace-coach-menubar" "${INSTALL_DIR}/pace-coach-menubar"
echo "    Installed: ${INSTALL_DIR}/pace-coach-menubar"

rm -rf "$TMP"

echo ""
echo "Done! pace-coach from branch '${BRANCH}' (run ${RUN_ID}) installed."
echo "Run 'pace-coach start' to begin."
echo "To go back to the stable release: curl -fsSL https://raw.githubusercontent.com/${REPO}/main/install.sh | sh"
