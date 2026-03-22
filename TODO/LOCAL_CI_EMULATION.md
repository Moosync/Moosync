# Local CI Emulation Setup

## Overview
When CI becomes problematic, set up local emulation using `act` to debug builds without waiting for GitHub Actions.

## Why
- Rust/Gradle/Tauri tooling is complex - CI debugging happens often
- Want exact GH:A parity for reproducible debugging
- Keep disk limit (14GB) to catch storage issues, but let CPU/RAM passthrough for speed

## Prerequisites
```bash
# Install act
brew install act  # macOS
# or download from https://github.com/nektos/act/releases

# Ensure Docker is running
```

## Dockerfile (`.github/Dockerfile.ci`)

```dockerfile
FROM ghcr.io/actions/ubuntu:22.04

# All apt packages from build.yaml + tests.yaml
RUN apt-get update && apt-get install -y \
    libwebkit2gtk-4.1-dev gir1.2-javascriptcoregtk-4.1 \
    libgtk-3-dev libunwind-dev librsvg2-dev patchelf \
    alsa-tools libasound2-dev libudev-dev \
    libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev libgstreamer-plugins-bad1.0-dev \
    gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad \
    gstreamer1.0-plugins-ugly gstreamer1.0-libav gstreamer1.0-tools \
    gstreamer1.0-x gstreamer1.0-alsa gstreamer1.0-gl \
    gstreamer1.0-gtk3 gstreamer1.0-qt5 gstreamer1.0-pulseaudio \
    libappindicator3-dev libayatana-appindicator3-dev libdbus-1-dev \
    nasm yasm autoconf automake libtool pkg-config \
    make build-essential binutils lld clang libenchant-2-2 \
    curl file xdg-utils libfuse2 jq && \
    rm -rf /var/lib/apt/lists/*

# Install Bazel
RUN curl -fsSL https://bazel.build/bazel-release.pub.gpg | gpg --dearmor > /usr/share/keyrings/bazel.gpg && \
    echo "deb [arch=amd64 signed-by=/usr/share/keyrings/bazel.gpg] https://storage.googleapis.com/bazel-apt stable jdk17" \
    | tee /etc/apt/sources.list.d/bazel.list && \
    apt-get update && apt-get install -y bazel && rm -rf /var/lib/apt/lists/*

# Install Java 17 (matching actions/setup-java with Zulu)
RUN curl -sL "https://api.adoptium.net/v3/binary/latest/17/ga/linux/x64/jdk" \
    -o /tmp/jdk.tar.gz && \
    tar -xzf /tmp/jdk.tar.gz -C /usr/lib/jvm && \
    ln -s /usr/lib/jvm/temurin-17+/bin/java /usr/bin/java && \
    rm /tmp/jdk.tar.gz

# Install Android SDK (matching GH:A's /usr/local/lib/android/sdk)
ENV ANDROID_HOME=/usr/local/lib/android/sdk
ENV ANDROID_SDK_ROOT=/usr/local/lib/android/sdk

RUN yes | sdkmanager --install "platform-tools" "platforms;android-34" \
    "build-tools;34.0.0" "ndk;26.1.10909125" "cmdline-tools;latest" 2>/dev/null || true

ENV JAVA_HOME=/usr/lib/jvm/temurin-17+
ENV PATH="${ANDROID_HOME}/cmdline-tools/latest/bin:${ANDROID_HOME}/platform-tools:${JAVA_HOME}/bin:$PATH"

# Set GH:A-like environment
ENV GITHUB_ACTIONS=true
ENV RUNNER_OS=Linux
ENV RUNNER_ARCH=x64
```

## Build the image

```bash
cd .github
docker build -t moosync/ci-ubuntu:latest -f Dockerfile.ci .
```

## Run act for specific jobs

### Linux x64 build (includes Android APKs)
```bash
act -j build-and-test \
  -P ubuntu-22.04=moosync/ci-ubuntu:latest \
  --container-architecture linux/amd64 \
  --container-options="--storage-opt=size=14g" \
  -e WORKFLOW_PLATFORM=linux
```

### All platforms (Linux only)
```bash
act -j build-and-test \
  -P ubuntu-22.04=moosync/ci-ubuntu:latest \
  --container-architecture linux/amd64 \
  --container-options="--storage-opt=size=14g"
```

### Tests only
```bash
act -j test \
  -P ubuntu-22.04=moosync/ci-ubuntu:latest \
  --container-architecture linux/amd64
```

## Notes

- **Storage limit**: `--storage-opt=size=14g` matches GH:A free tier limit to catch disk exhaustion
- **CPU/RAM**: Not limited - passthrough to local hardware for faster builds
- **Base image**: Uses `ghcr.io/actions/ubuntu:22.04` for exact GH:A parity
- **Event file**: `WORKFLOW_PLATFORM` env var can be used to filter platforms if needed

## Troubleshooting

### If `--storage-opt` doesn't work
```bash
# Check Docker support
docker info | grep -i storage

# Alternative: monitor manually
docker stats --no-stream <container-id>
```

### Verify pre-installed tools
```bash
docker run --rm moosync/ci-ubuntu:latest bash -c "
  echo '=== Java ===' && java -version 2>&1
  echo '=== Bazel ===' && bazel --version 2>&1
  echo '=== Android SDK ===' && echo \$ANDROID_HOME && ls \$ANDROID_HOME
"
```

### Clean up
```bash
docker image rm moosync/ci-ubuntu:latest
```
