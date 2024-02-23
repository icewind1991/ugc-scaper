{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-23.11";
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.flake-utils.follows = "utils";
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    naersk,
    rust-overlay,
  }:
    utils.lib.eachDefaultSystem (system: let
      overlays = [
        (import rust-overlay)
        (import ./overlay.nix)
      ];
      pkgs = (import nixpkgs) {
        inherit system overlays;
      };
      inherit (pkgs) lib callPackage rust-bin mkShell;
      inherit (lib.sources) sourceByRegex;
      inherit (builtins) fromTOML readFile;

      msrv = (fromTOML (readFile ./Cargo.toml)).package.rust-version;
      toolchain = rust-bin.stable.latest.default;
      msrvToolchain = rust-bin.stable."${msrv}".default;

      naersk' = callPackage naersk {
        rustc = toolchain;
        cargo = toolchain;
      };
      msrvNaersk = callPackage naersk {
        rustc = msrvToolchain;
        cargo = msrvToolchain;
      };

      src = sourceByRegex ./. ["Cargo.*" "(src|derive|benches|tests|examples)(/.*)?"];

      deps = with pkgs; [
        pkg-config
        openssl
      ];

      nearskOpt = {
        pname = "ugc-scraper";
        root = src;
        nativeBuildInputs = deps;
      };
    in rec {
      packages = rec {
        check = naersk'.buildPackage (nearskOpt
          // {
            mode = "check";
          });
        clippy = naersk'.buildPackage (nearskOpt
          // {
            mode = "clippy";
          });
        test = naersk'.buildPackage (nearskOpt
          // {
            release = false;
            mode = "test";
          });
        msrv = msrvNaersk.buildPackage (nearskOpt
          // {
            mode = "check";
          });
        inherit (pkgs) ugc-api-server;
        default = ugc-api-server;
      };

      devShells = let
        tools = with pkgs; [
          bacon
          cargo-insta
          cargo-edit
          cargo-outdated
          cargo-audit
          cargo-msrv
          cargo-semver-checks
        ];
      in {
        default = mkShell {
          OPENSSL_NO_VENDOR = 1;
          nativeBuildInputs = [toolchain] ++ tools ++ deps;
        };
        msrv = mkShell {
          OPENSSL_NO_VENDOR = 1;
          nativeBuildInputs = [msrvToolchain] ++ tools ++ deps;
        };
      };
    })
    // {
      overlays.default = import ./overlay.nix;
      nixosModules.default = {
        pkgs,
        config,
        lib,
        ...
      }: {
        imports = [./module.nix];
        config = lib.mkIf config.services.ugc-api-server.enable {
          nixpkgs.overlays = [self.overlays.default];
          services.ugc-api-server.package = lib.mkDefault pkgs.ugc-api-server;
        };
      };
    };
}
