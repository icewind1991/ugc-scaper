final: prev: {
  ugc-api-server = final.callPackage ./package.nix { };
  ugc-api-archiver = final.callPackage ./archiver.nix { };
}
