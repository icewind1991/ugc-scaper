{ rustPlatform
, openssl
, pkg-config
, lib
,
}:
let
  inherit (lib.sources) sourceByRegex;
  inherit (builtins) fromTOML readFile;
  src = sourceByRegex ./api-server [ "Cargo.*" "(src)(/.*)?" "README.md" ];
  version = (fromTOML (readFile api-server/Cargo.toml)).package.version;
in
rustPlatform.buildRustPackage rec {
  pname = "ugc-api-server";

  inherit src version;

  buildInputs = [
    openssl
  ];

  nativeBuildInputs = [
    pkg-config
  ];

  OPENSSL_NO_VENDOR = 1;

  doCheck = false;

  cargoLock = {
    lockFile = ./api-server/Cargo.lock;
  };
}
