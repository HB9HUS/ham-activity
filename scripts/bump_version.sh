#!/usr/bin/env bash
# -------------------------------------------------------------
#  bump‑version.sh
#
#  Usage:
#      ./bump-version.sh [major|minor|patch]   # defaults to patch
#
# -------------------------------------------------------------

set -euxo pipefail

# ---------- Helper: usage -------------------------------------------------
usage() {
    cat <<EOF >&2
Usage: $(basename "$0") [major|minor|patch]

  major   – increment the first number, reset minor & patch to 0
  minor   – increment the second number, reset patch to 0
  patch   – increment the third number (default)

Examples:
  $(basename "$0")            # bump patch → v1.2.4 → v1.2.5
  $(basename "$0") minor      # bump minor → v1.2.4 → v1.3.0
  $(basename "$0") major      # bump major → v1.2.4 → v2.0.0
EOF
    exit 1
}

# ---------- Parse the optional argument ------------------------------------
# Default is “patch”
bump_part="patch"

if [[ $# -gt 1 ]]; then
    usage
elif [[ $# -eq 1 ]]; then
    case "$1" in
        major|minor|patch) bump_part="$1" ;;
        *) usage ;;
    esac
fi

# ---------- 1️⃣ Find the newest tag that starts with "v" --------------------
# If no tag exists we start from v0.0.0
last_tag=$(git tag --list 'v*' --sort=-v:refname | head -n1 || true)

if [[ -z "$last_tag" ]]; then
    echo "No existing v* tag found – starting from v0.0.0"
    major=0
    minor=0
    patch=0
else
    # ---------- 2️⃣ Strip leading “v” and split into components -------------
    #   Example: v2.4.7 → major=2, minor=4, patch=7
    IFS='.' read -r major minor patch <<<"${last_tag#v}"
fi

# ---------- 3️⃣ Increment the requested component ------------------------
case "$bump_part" in
    major)
        major=$((major + 1))
        minor=0
        patch=0
        ;;
    minor)
        minor=$((minor + 1))
        patch=0
        ;;
    patch)
        patch=$((patch + 1))
        ;;
esac

# ---------- 4️⃣ Assemble the new tag --------------------------------------
new_tag="v${major}.${minor}.${patch}"

# ---------- 5️⃣ Create an annotated tag on HEAD --------------------------
git tag -a "${new_tag}" -m "Bump version to ${new_tag}"

# ---------- 6️⃣ (Optional) push the tag to the remote --------------------
# Uncomment the line below if you want the script to push automatically.
# git push origin "${new_tag}"

echo "Created tag ${new_tag}"
