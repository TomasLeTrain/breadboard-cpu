# shell.nix
let
  pkgs = import <nixpkgs> {};
  inputs.nixpkgs.url = "github:nixos/nixpkgs";
in
  pkgs.mkShell {
    packages = [
      (pkgs.python3.withPackages (python-pkgs:
        with python-pkgs; [
          # select Python packages here
					pillow
        ]))
    ];
  }
