#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck disable=SC1091
source "$ROOT/tool/env.sh"

if ! "$ANDROID_HOME/platform-tools/adb" devices | grep -q 'emulator-.*device'; then
  if "$ANDROID_HOME/emulator/emulator" -list-avds | grep -qx candil; then
    nohup "$ANDROID_HOME/emulator/emulator" -avd candil -no-audio -no-boot-anim \
      > "/tmp/candil-emulator.log" 2>&1 &
    "$ANDROID_HOME/platform-tools/adb" wait-for-device
    for _ in $(seq 1 120); do
      if [[ "$("$ANDROID_HOME/platform-tools/adb" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" == "1" ]]; then
        break
      fi
      sleep 2
    done
  fi
fi

"$ROOT/tool/build_android_rust.sh"
cd "$ROOT"
flutter run
