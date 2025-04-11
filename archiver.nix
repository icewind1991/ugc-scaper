{ rustPlatform
, openssl
, pkg-config
, lib
,
}:
let
  inherit (lib.sources) sourceByRegex;
  inherit (builtins) fromTOML readFile;
  src = sourceByRegex ./archiver [ "Cargo.*" "(src)(/.*)?" "README.md" "(.sqlx)(/.*)?"  ];
  version = (fromTOML (readFile archiver/Cargo.toml)).package.version;
in
rustPlatform.buildRustPackage rec {
  pname = "ugc-api-archiver";

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
    lockFile = ./archiver/Cargo.lock;
  };
}
