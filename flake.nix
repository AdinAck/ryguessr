{
  description = "ryguessr — Next.js frontend + Rust backend";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # ── Frontend (Next.js) ──────────────────────────────────────
        frontend = pkgs.buildNpmPackage {
          pname = "web-ryguessr";
          version = "0.1.0";
          src = ./web;

          npmDepsHash = "sha256-YvRlBzeEzyxRiIi5D66887PSIP0u81hriddd5Ji/3G4=";

          buildPhase = ''
            runHook preBuild
            npm run build
            runHook postBuild
          '';

          installPhase = ''
            runHook preInstall
            mkdir -p $out
            cp -r out/* $out/
            runHook postInstall
          '';
        };

        # ── Backend (Rust) ──────────────────────────────────────────
        commonArgs = {
          pname = "ryguessr";
          version = "0.1.0";
          src = craneLib.cleanCargoSource ./ryguessr;
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        server = craneLib.buildPackage (commonArgs
          // {
            inherit cargoArtifacts;
            FRONTEND_DIST = "${frontend}";
          });
      in {
        packages = {
          inherit frontend server;
          default = server;
        };

        # devshell with `nix develop`
        devShells.default = craneLib.devShell {
          packages = with pkgs; [
            # JS / frontend
            nodejs
            nodePackages.npm

            # Rust extras
            rust-analyzer
            cargo-watch

            # Tools
            just
          ];

          FRONTEND_DIST = "./web/out";
        };

        checks = {
          # Cargo tests
          server-test = craneLib.cargoTest (commonArgs
            // {
              inherit cargoArtifacts;
            });

          # Clippy
          server-clippy = craneLib.cargoClippy (commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- -D warnings";
            });

          # Cargo fmt
          server-fmt = craneLib.cargoFmt {
            src = craneLib.cleanCargoSource ./ryguessr;
          };
        };
      }
    );
}
