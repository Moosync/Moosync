#!/usr/bin/env bash
set -euo pipefail

ANDROID_HOME="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}}"
export ANDROID_HOME
PATH="$ANDROID_HOME/emulator:$ANDROID_HOME/platform-tools:$PATH"

APK_FILE=$(find . -name "moosync_apk.apk" -o -name "moosync_apk_unsigned.apk" 2>/dev/null | head -n 1)
if [ -z "$APK_FILE" ]; then
    echo "Error: APK not found in runfiles" >&2
    exit 1
fi

RUNNING_DEV=$(adb devices | grep -E "emulator-[0-9]+" | head -n 1 | awk '{print $1}' || true)
if [ -z "$RUNNING_DEV" ]; then
    AVD_NAME=$(emulator -list-avds 2>/dev/null | head -n 1 || true)
    if [ -z "$AVD_NAME" ]; then
        echo "Error: No AVD found. Please create an AVD via Android Studio or avdmanager." >&2
        exit 1
    fi
    echo "Starting Android emulator with AVD: $AVD_NAME..."
    nohup emulator -avd "$AVD_NAME" -netdelay none -netspeed full > /dev/null 2>&1 &

    echo "Waiting for emulator device..."
    adb wait-for-device
    while [ "$(adb shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" != "1" ]; do
        sleep 2
    done
    echo "Emulator is online and booted!"
fi

echo "Installing APK: $APK_FILE..."
adb install -r -g "$APK_FILE"

echo "Clearing logcat and launching Moosync Mobile..."
adb logcat -c || true
adb shell am start -W -n app.moosync.android/app.moosync.android.MainActivity

MOOSYNC_PID=$(adb shell pidof -s app.moosync.android 2>/dev/null | tr -d '\r' || true)

if [ "${1:-}" = "--dump" ]; then
    sleep 3
    if [ -n "$MOOSYNC_PID" ]; then
        adb logcat -d --pid="$MOOSYNC_PID"
    else
        adb logcat -d -s MoosyncAndroid:V MoosyncAndroidRust:V RustStdoutStderr:V AndroidRuntime:E
    fi
    exit 0
fi

if [ -n "$MOOSYNC_PID" ]; then
    echo "Moosync launched (PID: $MOOSYNC_PID)! Streaming Moosync logs (Ctrl+C to stop)..."
    adb logcat -v color --pid="$MOOSYNC_PID"
else
    echo "Moosync launched! Streaming Moosync logs (Ctrl+C to stop)..."
    adb logcat -v color -s MoosyncAndroid:V MoosyncAndroidRust:V RustStdoutStderr:V AndroidRuntime:E
fi
