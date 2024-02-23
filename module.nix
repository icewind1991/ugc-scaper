{
  config,
  lib,
  pkgs,
  ...
}:
with lib; let
  cfg = config.services.ugc-api-server;
in {
  options.services.ugc-api-server = {
    enable = mkEnableOption "ugc api server";

    logLevel = mkOption {
      type = types.str;
      default = "INFO";
      description = "log level";
    };

    port = mkOption {
      type = types.port;
      default = 10333;
      description = "port to listen to";
    };

    package = mkOption {
      type = types.package;
      description = "package to use";
    };
  };

  config = mkIf cfg.enable {
    systemd.services."ugc-api-server" = {
      wantedBy = ["multi-user.target"];
      after = ["network-online.target"];
      wants = ["network-online.target"];
      environment = {
        RUST_LOG = cfg.logLevel;
        PORT = toString  cfg.port;
      };

      serviceConfig = {
        ExecStart = "${cfg.package}/bin/ugc-api-server";
        Restart = "on-failure";
        DynamicUser = true;
        PrivateTmp = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        NoNewPrivileges = true;
        PrivateDevices = true;
        ProtectClock = true;
        CapabilityBoundingSet = true;
        ProtectKernelLogs = true;
        ProtectControlGroups = true;
        SystemCallArchitectures = "native";
        ProtectKernelModules = true;
        RestrictNamespaces = true;
        MemoryDenyWriteExecute = true;
        ProtectHostname = true;
        LockPersonality = true;
        ProtectKernelTunables = true;
        RestrictAddressFamilies = "AF_INET AF_INET6";
        RestrictRealtime = true;
        ProtectProc = "noaccess";
        SystemCallFilter = ["@system-service" "~@resources" "~@privileged"];
        PrivateUsers = true;
        ProcSubset = "pid";
      };
    };
  };
}
