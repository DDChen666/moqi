#!/bin/sh
# Build a release DMG signed with the fixed release identity.
#
#   sh scripts/make_release_dmg.sh            # uses "Moqi Release"
#   sh scripts/make_release_dmg.sh "My Name"  # any identity from signing_identity.sh
#
# Why not `tauri build --bundles dmg`:
# - A checkout on exFAT (or any volume without Unix permissions) gives the
#   app 700 permissions. Dragged into Applications like that, it vanishes
#   from Launchpad and Spotlight. The app is staged on the system disk and
#   opened up to go+rX first; permissions are not part of the signature.
# - Tauri's DMG step lays out the window by driving Finder with AppleScript,
#   which needs an Automation permission and fails without one. A plain
#   hdiutil image with an Applications link needs nothing.
set -e
IDENTITY="${1:-Moqi Release}"
cd "$(dirname "$0")/.."
VERSION=$(sed -n 's/.*"version": *"\([^"]*\)".*/\1/p' src-tauri/tauri.conf.json | head -1)
OUT="src-tauri/target/release/bundle/dmg/Moqi_${VERSION}_aarch64.dmg"

sh scripts/signing_identity.sh "$IDENTITY" >/dev/null
APPLE_SIGNING_IDENTITY="$IDENTITY" bun run tauri build --bundles app

STAGE=$(mktemp -d)
chmod 755 "$STAGE" # mktemp makes it 700; it becomes the volume root
trap 'rm -rf "$STAGE"' EXIT
ditto src-tauri/target/release/bundle/macos/Moqi.app "$STAGE/Moqi.app"
chmod -R go+rX "$STAGE/Moqi.app"
codesign --verify --deep --strict "$STAGE/Moqi.app"

ln -s /Applications "$STAGE/Applications"

mkdir -p "$(dirname "$OUT")"
rm -f "$OUT"
hdiutil create -volname Moqi -srcfolder "$STAGE" -fs HFS+ -format UDZO -ov "$OUT" >/dev/null
hdiutil verify "$OUT" >/dev/null
shasum -a 256 "$OUT"
