#!/bin/sh
# Install the rewire binary from GitHub releases: no Rust toolchain, no GitHub login.
#
#   curl -fsSL https://raw.githubusercontent.com/NewbeeTryToCode/rewire/main/scripts/install.sh | sh
#
# REWIRE_VERSION picks a release tag (default: the latest), REWIRE_INSTALL_DIR where the binary goes (default:
# ~/.local/bin), REWIRE_DOWNLOAD_URL a mirror of the release files. The download is checked against the release's SHA-256 file before anything is installed.
set -eu

REPO="NewbeeTryToCode/rewire"
VERSION="${REWIRE_VERSION:-latest}"
DIR="${REWIRE_INSTALL_DIR:-$HOME/.local/bin}"

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64 | Linux-amd64) target=x86_64-unknown-linux-musl ;;
  Darwin-arm64) target=aarch64-apple-darwin ;;
  Darwin-x86_64) target=x86_64-apple-darwin ;;
  *)
    echo "rewire: no release binary for $(uname -s) $(uname -m); build it with cargo (see the README)" >&2
    exit 1
    ;;
esac

if [ -n "${REWIRE_DOWNLOAD_URL:-}" ]; then
  base="$REWIRE_DOWNLOAD_URL" # a mirror holding the same release files
elif [ "$VERSION" = latest ]; then
  base="https://github.com/$REPO/releases/latest/download"
else
  base="https://github.com/$REPO/releases/download/$VERSION"
fi
asset="rewire-$target.tar.gz"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT INT TERM
curl -fsSL --retry 3 -o "$tmp/$asset" "$base/$asset"
curl -fsSL --retry 3 -o "$tmp/$asset.sha256" "$base/$asset.sha256"
if command -v sha256sum >/dev/null 2>&1; then
  (cd "$tmp" && sha256sum -c "$asset.sha256" >/dev/null)
else
  (cd "$tmp" && shasum -a 256 -c "$asset.sha256" >/dev/null)
fi
tar -xzf "$tmp/$asset" -C "$tmp"
mkdir -p "$DIR"
cp "$tmp/rewire-$target/rewire" "$DIR/rewire.new"
chmod 755 "$DIR/rewire.new"
mv -f "$DIR/rewire.new" "$DIR/rewire" # in place in one step: a running hook keeps the old file
echo "$("$DIR/rewire" --version) installed to $DIR/rewire"
case ":$PATH:" in
  *":$DIR:"*) ;;
  *) echo "add $DIR to your PATH to run \`rewire\` by name" ;;
esac
