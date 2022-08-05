{ pkgs }:
(import ./Cargo.nix { inherit pkgs; }).rootCrate.build
