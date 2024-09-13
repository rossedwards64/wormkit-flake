{
  description = "Development environment for creating WormKit modules on Linux, in Rust.";

  inputs = {
    nixpkgs.url = "github:nixOS/nixpkgs/master";
    flake-utils.url = "github:numtide/flake-utils";

    msvc-wine-rust = {
      url = "github:est31/msvc-wine-rust";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      msvc-wine-rust,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        toolchain = "stable";
        host = "x86_64-unknown-linux-gnu";
        target = "i686-pc-windows-msvc";

        pkgsCrossMingw = pkgs.pkgsCross.mingw32;
        mingw_w64 = pkgsCrossMingw.windows.mingw_w64;
        mingw_w64_pthreads = pkgsCrossMingw.windows.pthreads.overrideAttrs (oldAttrs: {
          configureFlags = (oldAttrs.configureFlags or [ ]) ++ [ "--enable-static" ];
        });

        msvcBaseDir = "/tmp/msvc";
        msvcDownloadDir = "${msvcBaseDir}/dl";
        msvcExtractedDir = "${msvcBaseDir}/extracted";
        acceptedLicensesFile = "${pkgs.writeText "licenses-accepted" ""}";

        msvcDownloadSha256 = pkgs.writeText "dl.sha256" (
          builtins.replaceStrings [ "dl" ] [ "${msvcDownloadDir}" ] (
            builtins.readFile "${msvc-wine-rust}/dl.sha256"
          )
        );

        dlScript = pkgs.writeShellApplication {
          name = "get";
          runtimeInputs = [ pkgs.wget ];
          excludeShellChecks = [ "SC2068" ];
          extraShellCheckFlags = [ "--severity=error" ];
          text = (
            builtins.replaceStrings
              [
                "#!/usr/bin/env bash"
                "-P dl"
                "mkdir -p dl"
                "mkdir -p extracted"
                "-so dl/"
                "> extracted"
                "-f licenses-accepted"
                "touch licenses-accepted"
                "extracted/"
                "dl.sha256"
                "dl/microsoft"
                "dl/Windows"
                "dl/Universal"
              ]
              [
                ""
                "-P ${msvcDownloadDir}"
                "mkdir -p ${msvcDownloadDir}"
                "mkdir -p ${msvcExtractedDir}"
                "-so ${msvcDownloadDir}/"
                "> ${msvcExtractedDir}"
                "-f ${acceptedLicensesFile}"
                ":" # noop
                "${msvcExtractedDir}/"
                "${msvcDownloadSha256}"
                "${msvcDownloadDir}/microsoft"
                "${msvcDownloadDir}/Windows"
                "${msvcDownloadDir}/Universal"
              ]
              (builtins.readFile "${msvc-wine-rust}/get.sh")
          );
        };

        linkerScript = pkgs.writeShellApplication {
          name = "linker";
          runtimeInputs = [ pkgs.wine ];
          excludeShellChecks = [ "SC2068" ];
          extraShellCheckFlags = [ "--severity=error" ];
          text = (
            builtins.replaceStrings
              [
                "#!/usr/bin/env bash"
                "$( cd \"$( dirname \"\${BASH_SOURCE[0]}\" )\" && cd ../extracted && pwd )"
                "=x64"
              ]
              [
                ""
                "${msvcExtractedDir}"
                "=x86"
              ]
              (builtins.readFile "${msvc-wine-rust}/linker-scripts/linker.sh")
          );
        };
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            cargo
            msitools
            pkgsCrossMingw.stdenv.cc
            rustup
            wine
          ];

          WINEARCH = "win32";
          RUSTUP_TOOLCHAIN = toolchain;
          CARGO_BUILD_TARGET = target;
          CARGO_TARGET_I686_PC_WINDOWS_MSVC_RUNNER = "${pkgs.wine}/bin/wine";
          CARGO_TARGET_I686_PC_WINDOWS_MSVC_LINKER = "${linkerScript}/bin/linker";
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

          RUSTFLAGS =
            (builtins.map (path: "-L ${path}/lib") [
              mingw_w64
              mingw_w64_pthreads
            ])
            ++ [ "-Clink-arg=/DEBUG:NONE" ];

          shellHook = ''
            set -o nounset
            set -o pipefail

            proj_dir=$(pwd)
            if [ ! -d ${msvcBaseDir} ]
            then
                pushd "${msvc-wine-rust}"
                ${dlScript}/bin/get licenses-accepted
                popd
            fi

            # taken from https://github.com/lucasew/nixcfg/blob/master/nix/pkgs/wrapWine.nix
            HOME="$(echo ~)"
            export WINEPREFIX="$HOME/.local/share/wineprefixes/default32"
            mkdir -p "$WINEPREFIX"
            if [ ! -d "$WINEPREFIX" ]
            then
                wine cmd /c dir > /dev/null 2> /dev/null
                wineserver -w
            fi
            export PATH=$PATH:$RUSTUP_HOME/toolchains/${toolchain}-${host}/bin/
            rustup target add "${target}"
          '';
        };
      }
    );
}
