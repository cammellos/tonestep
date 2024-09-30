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

      androidComposition = pkgs.androidenv.composeAndroidPackages {
        abiVersions = [ "arm64-v8a" "x86_64" ];
        includeNDK = true;
        platformVersions = [ "30" ];
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
              pcre2.dev
              rustup
            ];

        nativeBuildInputs = [
          pkgs.pkg-config
          pkgs.rustup
          pkgs.cargo-apk
          pkgs.jdk
          pkgs.alsa-lib
        ];

        shellHook = ''
          export SSL_CERT_FILE="${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
          export ANDROID_SDK_ROOT="${androidComposition.androidsdk}/libexec/android-sdk"
          export ANDROID_NDK_ROOT="$ANDROID_SDK_ROOT/ndk-bundle"
          export PATH="$HOME/.cargo/bin:$PATH"

          export LD_LIBRARY_PATH="$(pwd)/eartrainer/build/linux/x64/debug/bundle/lib:$LD_LIBRARY_PATH"

          cargo install flutter_rust_bridge_codegen
        '';

      };
    });
}
