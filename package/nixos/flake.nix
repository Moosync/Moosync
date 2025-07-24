{
  description = "Moosync is a simple music player with a primary goal to provide a clean and easy interface.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        pname = "moosync";
        version = "11.0.2";
        arch =
          if system == "x86_64-linux" then "amd64"
          else if system == "aarch64-linux" then "arm64"
          else throw "Unsupported system: ${system}. Try building moosync-source";
        src = pkgs.fetchurl {
          url = "https://github.com/Moosync/Moosync/releases/download/Moosync-v${version}/Moosync_${version}_${arch}.deb";
          sha256 =
            if arch == "amd64" then "fcca027b8c3c5d28f18cf9641f61f4db008cce7dee6de17f207732a8f549b1ac"
            else if arch == "arm64" then "d4dca8194419d526182c59bb1fbcd7e772460bfb921ff024441b0f6eb3ab114b"
            else throw "Unsupported arch: ${arch}";
        };
        moosync = pkgs.stdenv.mkDerivation {
          name = "${pname}-${version}";
          src = src;
          nativeBuildInputs = [ pkgs.binutils pkgs.libarchive ];
          buildInputs = [
            pkgs.gtk3
            pkgs.webkitgtk_4_1
            pkgs.libappindicator-gtk3
            pkgs.librsvg
            pkgs.alsa-lib
            pkgs.gst_all_1.gstreamer
            pkgs.gst_all_1.gst-plugins-base
            pkgs.gst_all_1.gst-plugins-good
            pkgs.gst_all_1.gst-plugins-bad
            pkgs.gst_all_1.gst-plugins-ugly
          ];
          unpackPhase = ''
            ar x $src
            bsdtar -xf data.tar.gz
          '';
          installPhase = ''
            install -Dm755 usr/bin/moosync $out/bin/moosync
            install -Dm644 usr/share/applications/Moosync.desktop $out/share/applications/Moosync.desktop
            find usr/share/icons/hicolor -type f -name "*.png" -exec install -Dm644 {} $out/share/icons/hicolor/{} \;
          '';
          meta = with pkgs.lib; {
            description = "A simple music player";
            homepage = "https://github.com/Moosync/Moosync";
            license = licenses.gpl3;
            platforms = [ "x86_64-linux" "aarch64-linux" ];
            maintainers = [ "ovenoboyo" ];
          };
        };

        moosync-source = pkgs.stdenv.mkDerivation {
          name = "${pname}-source-${version}";
          src = pkgs.fetchFromGitHub {
            owner = "Moosync";
            repo = "Moosync";
            rev = "Moosync-v11.0.2";
            sha256 = "6ae6e8c57227a1418d2e644871ae6f577a2bc56cdc3762de04a7143166c31458";
          };
          nativeBuildInputs = [
            pkgs.pkg-config
            pkgs.cmake
            pkgs.rustc
            pkgs.cargo
            pkgs.python3
          ];
          buildInputs = [
            pkgs.gtk3
            pkgs.webkitgtk_4_1
            pkgs.libappindicator-gtk3
            pkgs.librsvg
            pkgs.alsa-lib
            pkgs.gst_all_1.gstreamer
            pkgs.gst_all_1.gst-plugins-base
            pkgs.gst_all_1.gst-plugins-good
            pkgs.gst_all_1.gst-plugins-bad
            pkgs.gst_all_1.gst-plugins-ugly
            pkgs.sqlite
            pkgs.openssl
          ];
          dontConfigure = true;
          buildPhase = ''
            export HOME=$PWD
            cargo install --locked trunk
            cargo install tauri-cli --version "^2.0.0-rc"
            ls -la
            cargo tauri build --no-bundle
          '';
          installPhase = ''
            install -Dm755 src-tauri/target/release/moosync $out/bin/moosync
            install -Dm644 extras/linux/Moosync.desktop $out/share/applications/Moosync.desktop
            install -Dm644 src-tauri/icons/32x32.png $out/share/icons/hicolor/32x32/apps/moosync.png
            install -Dm644 src-tauri/icons/128x128.png $out/share/icons/hicolor/128x128/apps/moosync.png
            install -Dm644 src-tauri/icons/icon.png $out/share/icons/hicolor/512x512/apps/moosync.png
          '';
          meta = with pkgs.lib; {
            description = "A simple music player (built from source)";
            homepage = "https://github.com/Moosync/Moosync";
            license = licenses.gpl3;
            platforms = [ "x86_64-linux" "aarch64-linux" ];
            maintainers = [ "ovenoboyo" ];
          };
        };
      in {
        packages.default = moosync;
        packages.moosync = moosync;
        packages.moosync-source = moosync-source;

        apps.default = flake-utils.lib.mkApp {
          drv = moosync;
          exePath = "/bin/moosync";
        };

        apps.moosync-source = flake-utils.lib.mkApp {
          drv = moosync-source;
          exePath = "/bin/moosync";
        };

        nixosModules.moosync = { config, pkgs, ... }: {
          environment.systemPackages = [ moosync ];
        };
        nixosModules.moosync-source = { config, pkgs, ... }: {
          environment.systemPackages = [ moosync-source ];
        };
      }
    );
}
