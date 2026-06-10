self: {
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.services.ryguessr;
  defaultServer = self.packages.${pkgs.system}.server;
  defaultWeb = self.packages.${pkgs.system}.web;
in {
  options.services.ryguessr = {
    enable = lib.mkEnableOption "ryguessr server";

    package = lib.mkOption {
      type = lib.types.package;
      default = defaultServer;
      defaultText = lib.literalExpression "ryguessr.packages.\${system}.server";
      description = "ryguessr server package to run.";
    };

    webPackage = lib.mkOption {
      type = lib.types.package;
      default = defaultWeb;
      defaultText = lib.literalExpression "ryguessr.packages.\${system}.web";
      description = ''
        Built Next.js static export to serve as the fallback. The server
        serves files from this path for any non-API route.
      '';
    };

    bindAddr = lib.mkOption {
      type = lib.types.str;
      default = "127.0.0.1:3000";
      example = "0.0.0.0:3000";
      description = ''
        Address and port to listen on. Default binds to loopback only —
        put a reverse proxy in front, or set 0.0.0.0:PORT to expose directly.
      '';
    };

    googleMapsApiKeyFile = lib.mkOption {
      type = lib.types.path;
      example = "/run/secrets/ryguessr_google_maps_api_key";
      description = ''
        Path to a file containing only the Google Maps API key. The contents
        are read into GOOGLE_MAPS_API_KEY at service start via systemd
        LoadCredential, so the file does not need to be world-readable —
        it just needs to be readable by root at unit-start time. Pair this
        with sops-nix or agenix.
      '';
    };

    osmDataDir = lib.mkOption {
      type = lib.types.str;
      default = "/var/lib/ryguessr/osm";
      description = ''
        Directory containing preprocessed `.roadpoints` files. Populate
        before first start, e.g.

            sudo -u ryguessr ryguessr-setup-osm --region europe

        or run `nix run github:youruser/ryguessr#setup-osm -- --region europe`
        and move the output into place.
      '';
    };

    user = lib.mkOption {
      type = lib.types.str;
      default = "ryguessr";
      description = "System user to run the service as.";
    };

    group = lib.mkOption {
      type = lib.types.str;
      default = "ryguessr";
      description = "System group to run the service as.";
    };

    openFirewall = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = ''
        Open the bound TCP port in the firewall. Only effective when
        bindAddr is reachable from outside the host.
      '';
    };

    extraEnvironment = lib.mkOption {
      type = lib.types.attrsOf lib.types.str;
      default = {};
      example = lib.literalExpression ''{ RUST_LOG = "debug"; }'';
      description = "Extra environment variables to set on the service.";
    };
  };

  config = lib.mkIf cfg.enable {
    users.users.${cfg.user} = {
      isSystemUser = true;
      group = cfg.group;
      home = "/var/lib/ryguessr";
      description = "ryguessr service user";
    };
    users.groups.${cfg.group} = {};

    systemd.services.ryguessr = {
      description = "ryguessr server";
      after = ["network-online.target"];
      wants = ["network-online.target"];
      wantedBy = ["multi-user.target"];

      environment =
        {
          BIND_ADDR = cfg.bindAddr;
          OSM_DATA_DIR = cfg.osmDataDir;
          WEB_DIR = "${cfg.webPackage}";
          RUST_LOG = "info";
        }
        // cfg.extraEnvironment;

      serviceConfig = {
        User = cfg.user;
        Group = cfg.group;
        StateDirectory = "ryguessr";
        WorkingDirectory = "/var/lib/ryguessr";
        Restart = "on-failure";
        RestartSec = 5;

        # systemd reads the key file once at unit start and exposes it under
        # $CREDENTIALS_DIRECTORY/google_maps_api_key. The script wrapper
        # below loads it into the env without ever putting the secret on disk
        # or in the unit definition.
        LoadCredential = "google_maps_api_key:${cfg.googleMapsApiKeyFile}";

        # Hardening
        NoNewPrivileges = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        PrivateTmp = true;
        PrivateDevices = true;
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectControlGroups = true;
        RestrictAddressFamilies = ["AF_INET" "AF_INET6"];
        RestrictNamespaces = true;
        LockPersonality = true;
        MemoryDenyWriteExecute = true;
        SystemCallArchitectures = "native";
      };

      script = ''
        export GOOGLE_MAPS_API_KEY="$(cat "$CREDENTIALS_DIRECTORY/google_maps_api_key")"
        exec ${cfg.package}/bin/ryguessr
      '';
    };

    networking.firewall = lib.mkIf cfg.openFirewall {
      allowedTCPPorts = let
        parts = lib.splitString ":" cfg.bindAddr;
        port = lib.toInt (lib.last parts);
      in [port];
    };
  };
}
