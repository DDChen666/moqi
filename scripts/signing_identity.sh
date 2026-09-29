#!/bin/sh
# Create (once) and unlock a fixed self-signed code-signing identity.
#
#   sh scripts/signing_identity.sh "Moqi Release"
#   APPLE_SIGNING_IDENTITY="Moqi Release" bun run tauri build --bundles app,dmg
#
# Why: macOS ties the Microphone and Accessibility grants to the app's code
# signature. An ad-hoc signature changes on every build, so every rebuild or
# update would need the grants again. Signing every build with the same
# certificate keeps them.
#
# This is not an Apple Developer ID: Gatekeeper still blocks the first launch
# (see docs/安裝教學-Mac.md). It only keeps the identity stable.
#
# The identity lives in its own keychain under ~/.config/moqi-signing/<name>/,
# away from your login keychain, with a random password stored next to it.
# backup.p12 there (same password) is the only copy of the private key:
# back up that folder. Losing it means users must grant permissions once more.
# Safe to run again: it only unlocks an identity that already exists.
# To restore on a new Mac, copy backup.p12 and pass into that folder first.
set -e
NAME="${1:?usage: sh scripts/signing_identity.sh \"Identity Name\"}"
SLUG=$(printf '%s' "$NAME" | tr ' A-Z' '-a-z')
DIR="$HOME/.config/moqi-signing/$SLUG"
KC="$DIR/signing.keychain-db"
SSL=/usr/bin/openssl # macOS's own; Homebrew OpenSSL 3 writes p12 files `security` can't read

mkdir -p "$DIR" && chmod 700 "$DIR"
[ -f "$DIR/pass" ] || { $SSL rand -hex 24 > "$DIR/pass"; chmod 600 "$DIR/pass"; }
PASS=$(cat "$DIR/pass")

if [ ! -f "$KC" ]; then
  security create-keychain -p "$PASS" "$KC"
  security set-keychain-settings "$KC" # no auto-lock
fi
security unlock-keychain -p "$PASS" "$KC"

if ! security find-certificate -c "$NAME" "$KC" >/dev/null 2>&1; then
  # A backup.p12 already here (restored from a backup) is imported as is.
  if [ ! -f "$DIR/backup.p12" ]; then
  cat > "$DIR/cert.cnf" <<EOF
[req]
distinguished_name = dn
x509_extensions = ext
prompt = no
[dn]
CN = $NAME
[ext]
basicConstraints = critical, CA:false
keyUsage = critical, digitalSignature
extendedKeyUsage = critical, codeSigning
EOF
  $SSL req -x509 -newkey rsa:2048 -nodes -days 3650 -config "$DIR/cert.cnf" \
    -keyout "$DIR/key.pem" -out "$DIR/cert.pem" 2>/dev/null
  $SSL pkcs12 -export -inkey "$DIR/key.pem" -in "$DIR/cert.pem" -name "$NAME" \
    -out "$DIR/backup.p12" -passout "pass:$PASS"
  chmod 600 "$DIR/backup.p12"
  rm -f "$DIR/key.pem"
  echo "Created \"$NAME\". Back up $DIR (backup.p12 + pass) somewhere safe."
  fi
  security import "$DIR/backup.p12" -k "$KC" -P "$PASS" -T /usr/bin/codesign
  # Let codesign use the key without a prompt.
  security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$PASS" "$KC" >/dev/null
fi

# Add the keychain to the search list so codesign finds it (keep the others).
if ! security list-keychains -d user | grep -q "$KC"; then
  # shellcheck disable=SC2046
  security list-keychains -d user -s $(security list-keychains -d user | tr -d '"') "$KC"
fi

security find-identity -p codesigning "$KC" | grep "$NAME"
