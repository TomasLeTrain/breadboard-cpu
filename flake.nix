{
  description = "Nix devshells!";
  inputs.nixpkgs.url = "github:nixos/nixpkgs";

  outputs = {
    self,
    nixpkgs,
    ...
  }: let
    pkgs = nixpkgs.legacyPackages.x86_64-linux;
    customasm = pkgs.callPackage ./customasm.nix {};
  in {
    devShells.x86_64-linux.default = pkgs.mkShell {
      packages = [
        customasm
      ];
    };
  };
}
