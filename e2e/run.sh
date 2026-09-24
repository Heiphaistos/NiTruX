#!/usr/bin/env bash
# Lance les tests de bout en bout NiTruX sur la machine courante.
#   ./e2e/run.sh                 parcours de toutes les pages + scénarios sans effet
#   ./e2e/run.sh --privileged    + scénarios qui modifient le système (pkexec)
# À lancer en utilisateur NORMAL (pas root) : c'est le cas réel, et root masque
# les problèmes de droits.
set -euo pipefail
cd "$(dirname "$0")/.."

missing=()
command -v WebKitWebDriver >/dev/null || missing+=("webkit2gtk-driver (apt) / webkit2gtk4.1-devel (dnf)")
command -v tauri-driver >/dev/null || [ -x "$HOME/.cargo/bin/tauri-driver" ] || missing+=("tauri-driver : cargo install tauri-driver --locked")
python3 -c "from selenium.webdriver.remote import webdriver" 2>/dev/null || missing+=("selenium (et ses dépendances) : pip install --user selenium")
if [ ${#missing[@]} -gt 0 ]; then
  printf 'Manquant :\n'; printf '  - %s\n' "${missing[@]}"; exit 1
fi
[ "$(id -u)" = 0 ] && echo "Attention : lancé en root, les problèmes de droits d'un utilisateur normal ne seront pas vus."

APP="${NITRUX_APP:-src-tauri/target/debug/nitrux}"
if [ ! -x "$APP" ] || [ "${REBUILD:-0}" = 1 ]; then
  echo "Compilation de l'application (debug)..."
  npm ci --silent
  npx tauri build --debug --no-bundle
fi
export NITRUX_APP="$(realpath "$APP")"

cleanup() { kill "${DRIVER_PID:-}" "${XVFB_PID:-}" 2>/dev/null || true; }
trap cleanup EXIT
if [ -z "${DISPLAY:-}" ]; then
  command -v Xvfb >/dev/null || { echo "Pas d'écran : installez xvfb ou lancez depuis la session graphique."; exit 1; }
  n=99
  while [ -e "/tmp/.X$n-lock" ] || [ -e "/tmp/.X11-unix/X$n" ]; do n=$((n + 1)); done
  Xvfb ":$n" -screen 0 1400x900x24 >/dev/null 2>&1 & XVFB_PID=$!
  export DISPLAY=":$n"; sleep 1
  kill -0 "$XVFB_PID" 2>/dev/null || { echo "Xvfb n'a pas pu démarrer sur $DISPLAY."; exit 1; }
fi
TD="$(command -v tauri-driver || echo "$HOME/.cargo/bin/tauri-driver")"
"$TD" --port 4444 > e2e/out-driver.log 2>&1 & DRIVER_PID=$!
sleep 2

status=0
python3 e2e/sweep.py || status=1
python3 e2e/flows.py "$@" || status=1
exit $status
