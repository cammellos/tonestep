{
  description = "Flake for Android development with Rust and Cargo APK";

  inputs = {
     android-nixpkgs = {
      url = "github:tadfisher/android-nixpkgs";
    };
    #nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable"; # or a specific version/branch you prefer
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, android-nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs { inherit system;

        config = {
          android_sdk.accept_license = true;
          allowUnfree = true;
        };

      };
      buildToolsVersion = "34.0.0";

      androidComposition = pkgs.androidenv.composeAndroidPackages {
        abiVersions = [ "arm64-v8a" "x86_64" ];
        includeNDK = true;
        ndkVersion = "23.1.7779620";
        buildToolsVersions = [ buildToolsVersion "30.0.3" ];
        platformVersions = [ "33" "34" ];
      };

    in {
      devShell = pkgs.mkShell {
        # Workaround for https://github.com/NixOS/nixpkgs/issues/60919.
        hardeningDisable = [ "fortify" ];

        # Allow cargo to download crates (even inside `nix-shell --pure`).
            buildInputs = with pkgs; [
              flutter
              pkg-config
              gtk3.dev
              lerc.dev
            util-linux.dev
            libselinux.dev
            libthai.dev
            libepoxy.dev
            xorg.libXtst
            libdatrie.dev
            libxkbcommon.dev
            xorg.libXdmcp.dev
            pcre2.dev
            libsepol.dev

              alsa-lib
          aapt
              pcre2.dev
              rustup
            ];

        nativeBuildInputs = [
          pkgs.pkg-config
          pkgs.rustup
          pkgs.cargo-apk
          pkgs.jdk17
          pkgs.alsa-lib
        ];

        shellHook = ''
          export SSL_CERT_FILE="${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
          export ANDROID_SDK_ROOT="${androidComposition.androidsdk}/libexec/android-sdk"
          export ANDROID_NDK_ROOT="$ANDROID_SDK_ROOT/ndk-bundle"
          export PATH="$HOME/.cargo/bin:$PATH"

          export LD_LIBRARY_PATH="$(pwd)/eartrainer/build/linux/x64/debug/bundle/lib:$LD_LIBRARY_PATH"
          export GRADLE_OPTS="-Dorg.gradle.project.android.aapt2FromMavenOverride=${androidComposition.androidsdk}/libexec/android-sdk/build-tools/${buildToolsVersion}/aapt2";

          cargo install flutter_rust_bridge_codegen
        '';

      };
    });
}
