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
    (flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        };
        lib = pkgs.lib;

        rustToolchain = pkgs.rust-bin.stable.latest.default;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Source for the Rust build: workspace manifest + lockfile + member
        # crates. Keeps web/, osm/, and other directories out of the build
        # input so editing them doesn't invalidate the Rust cache.
        rustSrc = lib.cleanSourceWith {
          src = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              ./server
              ./colors
              ./macros
            ];
          };
          filter = path: type:
            (craneLib.filterCargoSources path type)
            # include the compile-time word lists at server/assets/*.txt
            || (lib.hasInfix "/assets/" path && lib.hasSuffix ".txt" path);
        };

        commonArgs = {
          src = rustSrc;
          strictDeps = true;
          pname = "ryguessr";
          version = "0.1.0";
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        ryguessr-server = craneLib.buildPackage (commonArgs
          // {
            inherit cargoArtifacts;
            cargoExtraArgs = "-p ryguessr";
          });

        ryguessr-setup-osm = craneLib.buildPackage (commonArgs
          // {
            inherit cargoArtifacts;
            pname = "ryguessr-setup-osm";
            cargoExtraArgs = "-p ryguessr --example setup_osm";
            # Examples land under target/release/examples/, so install by hand.
            installPhaseCommand = ''
              install -Dm755 target/release/examples/setup_osm \
                $out/bin/ryguessr-setup-osm
            '';
          });

        ryguessr-web = pkgs.buildNpmPackage {
          pname = "ryguessr-web";
          version = "0.1.0";
          src = ./web;

          npmDepsHash = "sha256-I5vQO1MzHal4GpgsEVTNhERH2Vgpf7rps1yARTK6fvQ=";

          NEXT_TELEMETRY_DISABLED = "1";

          # next.config.ts uses `output: "export"`, so `npm run build`
          # produces a static site in ./out
          installPhase = ''
            runHook preInstall
            mkdir -p $out
            cp -r out/. $out/
            runHook postInstall
          '';
        };
      in {
        packages = {
          server = ryguessr-server;
          setup-osm = ryguessr-setup-osm;
          web = ryguessr-web;
          default = ryguessr-server;
        };

        apps.setup-osm = {
          type = "app";
          program = "${ryguessr-setup-osm}/bin/ryguessr-setup-osm";
        };

        devShells.default = craneLib.devShell {
          packages = with pkgs; [
            # JS / frontend
            nodejs
            prettier

            # Rust extras
            rust-analyzer
            cargo-watch

            # Tools
            just
            watchexec
          ];
        };
      }
    ))
    // {
      nixosModules.default = import ./nix/module.nix self;
    };
}
