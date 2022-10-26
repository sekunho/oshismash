{ config, pkgs, lib, oshismash, ... }:
  let
    cfg = config.services.oshismash;
  in with lib; {
    options = {
      services.oshismash = {
        enable = mkOption {
          default = false;
          type = with types; bool;
          description = "Start the oshismash server for a user";
        };

        host = mkOption {
          default = "localhost";
          type = with types; str;
          description = "Host oshismash use for its redirects";
        };

        port = mkOption {
          default = "3000";
          type = with types; str;
          description = "Port number oshismash will run on";
        };

        dbHost = mkOption {
          default = "localhost";
          type = with types; str;
          description = "Host of database server";
        };

        dbName = mkOption {
          default = "oshismash_db";
          type = with types; uniq str;
          description = "Database name for oshismash";
        };

        dbUser = mkOption {
          default = "postgres";
          type = with types; uniq str;
          description = "Database user";
        };

        dbPort = mkOption {
          default = "5432";
          type = with types; str;
          description = "Port number of database server";
        };

        dbPasswordFile = mkOption {
          type = with types; uniq str;
          description = "Path to DB password file";
        };

        dbCACertFile = mkOption {
          default = "";
          type = with types; uniq str;
          description = "Path to DB CA certificate";
        };

        dbPoolSize = mkOption {
          default = "1";
          type = with types; uniq str;
          description = "Path to DB CA certificate";
        };
      };
    };

    config = mkIf cfg.enable {
      # services.postgresql = {
      #   enable = true;
      #   extraPlugins = with pkgs.postgresql14Packages; [ pgtap ];
      #   package = pkgs.postgresql_14;

      #   # FIXME: Should change this one lol
      #   authentication = pkgs.lib.mkOverride 14 ''
      #     local all all trust
      #     host all all ::1/128 trust
      #     host all all localhost trust
      #   '';
      # };

      systemd.services.oshismash = {
        wantedBy = [ "multi-user.target" ];
        after = [ "network.target" ];
        description = "Start the oshismash server";

        environment = mkMerge [
          {
            APP__PORT = "${cfg.port}";
            APP__HOST = "${cfg.host}";
            PG__DBNAME = "${cfg.dbName}";
            PG__HOST = "${cfg.dbHost}";
            PG__USER = "${cfg.dbUser}";
            PG__PORT = "${cfg.dbPort}";
            PG__PASSWORD_FILE = "${cfg.dbPasswordFile}";
            PG__POOL_SIZE = "${cfg.dbPoolSize}";
          }

          (mkIf ("${cfg.dbCACertFile}" != "") {
            PG__CA_CERT = "${cfg.dbCACertFile}";
          })
        ];

        serviceConfig = {
          Type = "simple";
          ExecStart = "${oshismash}/bin/oshismash";
        };
      };

      environment.systemPackages = [ oshismash ];
    };
  }
