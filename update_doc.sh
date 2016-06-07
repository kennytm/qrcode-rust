#!/usr/bin/env bash

# Generate documentation and commit into the gh-pages branch.

set -euo pipefail

VERSION=$(grep -w version -m 1 Cargo.toml)
COMMIT=$(git rev-parse HEAD)

rustup default nightly
cargo doc
git worktree add doc gh-pages
cd doc
git rm -r .
git reset HEAD .gitignore index.html
git checkout -- .gitignore index.html
mv ../target/doc/qrcode .
mv ../target/doc/*.{txt,woff,js,css} .
mkdir src
mv ../target/doc/src/qrcode src
git add .
git commit -m "Update doc for ${VERSION} (${COMMIT})"
cd ..
rm -rf doc
git worktree prune

