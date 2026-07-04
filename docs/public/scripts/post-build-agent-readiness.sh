#!/usr/bin/env bash
# Post-mkdocs: copy agent-readiness assets into site/ for Cloudflare Pages.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SITE="$ROOT/site"
SRC="$ROOT/agent-readiness"
DOCS="$ROOT/docs"

echo "→ agent-readiness: copying discovery files to site/"
cp "$SRC/llms.txt" "$SITE/"
cp "$SRC/auth.md" "$SITE/"
cp "$SRC/robots.txt" "$SITE/"
cp "$SRC/_headers" "$SITE/"

mkdir -p "$SITE/.well-known"
cp -r "$SRC/.well-known/"* "$SITE/.well-known/"
cp "$ROOT/../../.well-known/security.txt" "$SITE/.well-known/security.txt"

echo "→ agent-readiness: mirroring source markdown to site/markdown/"
mkdir -p "$SITE/markdown"
while IFS= read -r -d '' f; do
  rel="${f#$DOCS/}"
  dest="$SITE/markdown/$rel"
  mkdir -p "$(dirname "$dest")"
  cp "$f" "$dest"
done < <(find "$DOCS" -name '*.md' -print0)

echo "→ agent-readiness: done ($(find "$SITE/markdown" -name '*.md' | wc -l) markdown files)"