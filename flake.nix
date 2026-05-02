{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = {
    self,
    nixpkgs,
    flake-utils,
    fenix,
  }:
    flake-utils.lib.eachDefaultSystem
    (
      system: let
        pkgs = import nixpkgs { 
          inherit system;
          config.allowUnfree = true;
        };
        rust-toolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-Z8PetnKGSZjqRtodJ20XqBoTe2qNG0RaklrVW7AQ3JE=";
        };

        # helper function to merge a bunch of attrsets recursively
        merge = pkgs.lib.foldl (a: b: pkgs.lib.recursiveUpdate a b ) {};
      in (merge [
        {
          devShells.default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              # rust
              rust-toolchain

              # embedded rust
              probe-rs-tools flip-link picotool

              pkg-config
            ];
            buildInputs = with pkgs; [
              openssl
            ];
          };
          formatter = pkgs.alejandra;
        }
      ])
    );
}
