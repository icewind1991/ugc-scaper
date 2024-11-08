{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-24.05";
    flakelight = {
      url = "github:nix-community/flakelight";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    mill-scale = {
      url = "github:icewind1991/mill-scale";
      inputs.flakelight.follows = "flakelight";
    };
  };
  outputs = { mill-scale, ... }: mill-scale ./. {
    extraFilesRegex = [ ".*\.html" ];
    nixosModules = { outputs, ... }: {
      default =
        { pkgs
        , config
        , lib
        , ...
        }: {
          imports = [ ./module.nix ];
          config = lib.mkIf config.services.ugc-api-server.enable {
            nixpkgs.overlays = [ (import ./overlay.nix) ];
            services.ugc-api-server.package = lib.mkDefault pkgs.ugc-api-server;
          };
        };
    };
  };
}
