{
  lib,
  fetchFromGitHub,
  rustPlatform,
}:
rustPlatform.buildRustPackage rec {
  pname = "customasm";
  version = "v0.14.1";

  src = fetchFromGitHub {
    owner = "hlorenzi";
    repo = pname;
    rev = version;
    sha256 = "sha256-HJAIMSAxzyHJfn4Qau5THsndZQKP568kmFoD1gVDv7c=";
  };

  cargoHash = "sha256-PSK1KwjM1gyRaBdvsTMhR4T8lO+A3BHpbQHQW+H+Rw0=";
}
