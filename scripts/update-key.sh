#!/usr/bin/env bash
# Rustcord's release key: run once, on the release machine.
#
#   scripts/update-key.sh [private-key-file]
#
# The private half goes to a file outside the repository (by default
# ~/.config/rustcord-release/update-signing.key, 0600) and, through `gh secret
# set`, to the GitHub Actions secret RUSTCORD_UPDATE_SIGNING_KEY of
# Niiihuel/rustcord; it is never printed. The public half goes to
# crates/rustcord-update/release-key.pub, to commit: every Rustcord built from then
# on trusts only releases signed with this key.
#
# Keep a copy of the private file somewhere safe. Installed copies cannot
# be told of another key: losing it means reinstalling by hand.
set -euo pipefail

repo="$(cd "$(dirname "$0")/.." && pwd)"
public="$repo/crates/rustcord-update/release-key.pub"
default_key="$HOME/.config/rustcord-release/update-signing.key"
legacy_key="$HOME/.config/rune-release/update-signing.key"
if [ "$#" -eq 0 ] && [ -s "$legacy_key" ] && [ ! -e "$default_key" ]; then
  default_key="$legacy_key"
fi
key="${1:-$default_key}"
key="$(realpath -m "$key")"

case "$key" in
  "$repo"/*)
    echo "update-key: the private key must live outside the repository" >&2
    exit 1
    ;;
esac
if [ -s "$public" ]; then
  echo "update-key: $public already holds a key; installed copies trust only that one" >&2
  echo "update-key: to replace it on purpose, empty that file first" >&2
  exit 1
fi

mkdir -p "$(dirname "$key")"
chmod 700 "$(dirname "$key")"
# The private half is written by the tool itself (0600, never over a file
# that exists); only the public half reaches stdout.
(cd "$repo" && cargo run --quiet --locked -p rustcord-release -- keygen "$key") > "$public"
echo "update-key: public key written to crates/rustcord-update/release-key.pub: $(cat "$public")"
echo "update-key: private key in $key"

if command -v gh > /dev/null; then
  gh secret set RUSTCORD_UPDATE_SIGNING_KEY --repo Niiihuel/rustcord < "$key"
  echo "update-key: secret RUSTCORD_UPDATE_SIGNING_KEY set in Niiihuel/rustcord"
else
  echo "update-key: gh is not installed; set the secret with:" >&2
  echo "  gh secret set RUSTCORD_UPDATE_SIGNING_KEY --repo Niiihuel/rustcord < \"$key\"" >&2
fi
echo "update-key: now commit crates/rustcord-update/release-key.pub"
