{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" ];
      perSystem = { config, self', pkgs, lib, system, ... }:
        let
          runtimeDeps = with pkgs; [ pipewire.dev ];
          buildDeps = with pkgs; [
            pkg-config
            rustPlatform.bindgenHook
            llvmPackages.libclang.lib
          ];

          cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);

          rustPackage = features:
            (pkgs.makeRustPlatform {
              cargo = pkgs.rust-bin.stable.latest.minimal;
              rustc = pkgs.rust-bin.stable.latest.minimal;
            }).buildRustPackage {
              inherit (cargoToml.package) name version;
              src = ./.;
              cargoLock.lockFile = ./Cargo.lock;
              buildFeatures = features;
              buildInputs = runtimeDeps;
              nativeBuildInputs = buildDeps;

              LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
              # Uncomment if your cargo tests require networking or otherwise
              # don't play nicely with the Nix build sandbox:
              # doCheck = false;
            };

        in {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ (import inputs.rust-overlay) ];
          };
          packages.default = rustPackage "";
          packages.kbar = rustPackage "";
          devShells.default = pkgs.mkShell {
            nativeBuildInputs = buildDeps ++ pkgs.rustc;
            LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          };
        };
    };
}
