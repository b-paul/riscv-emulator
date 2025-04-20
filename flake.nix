{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
        sail_csim = pkgs.callPackage ./nix/sail-riscv.nix { arch = "RV64"; };
      in
      {
        packages.default = naersk-lib.buildPackage ./.;
        devShells.default = with pkgs; mkShell {
          buildInputs = [
              cargo rustc rustfmt pre-commit rustPackages.clippy python311Packages.riscof sail_csim
              (writeShellScriptBin "riscv64-unknown-elf-objdump" "exec riscv64-elf-objdump $@")
              (writeShellScriptBin "riscv64-unknown-elf-gcc" "exec riscv64-elf-gcc $@")
          ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
      }
    );
}
