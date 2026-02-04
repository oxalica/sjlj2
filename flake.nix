{
  inputs.rust-overlay = {
    url = "github:oxalica/rust-overlay";
    inputs.nixpkgs.follows = "nixpkgs";
  };
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      inherit (nixpkgs) lib;
      eachSystem = lib.genAttrs lib.systems.flakeExposed;
    in
    {
      devShells = eachSystem (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          rust-bin = rust-overlay.lib.mkRustBin { } pkgs;
        in
        {
          default = pkgs.mkShell {
            nativeBuildInputs = [
              (lib.lowPrio (
                rust-bin.stable.latest.default.override {
                  targets = [
                    "i686-unknown-linux-gnu"
                    "riscv64gc-unknown-linux-gnu"
                    "aarch64-unknown-linux-gnu"
                    "armv7-unknown-linux-gnueabihf"
                  ];
                  extensions = [
                    "rust-src"
                    "clippy"
                  ];
                }
              ))
              pkgs.qemu
            ];

            env = {
              CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER =
                lib.getExe
                  (import nixpkgs {
                    inherit system;
                    crossSystem = "i686-linux";
                  }).buildPackages.gcc;

              CARGO_TARGET_RISCV64GC_UNKNOWN_LINUX_GNU_LINKER = lib.getExe pkgs.pkgsCross.riscv64.buildPackages.gcc;
              CARGO_TARGET_RISCV64GC_UNKNOWN_LINUX_GNU_RUNNER = "qemu-riscv64";
              CARGO_TARGET_RISCV32GC_UNKNOWN_LINUX_GNU_LINKER = lib.getExe pkgs.pkgsCross.riscv32.buildPackages.gcc;
              CARGO_TARGET_RISCV32GC_UNKNOWN_LINUX_GNU_RUNNER = "qemu-riscv32";

              CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = lib.getExe pkgs.pkgsCross.aarch64-multiplatform.buildPackages.gcc;
              CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER = "qemu-aarch64";
              CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER = lib.getExe pkgs.pkgsCross.armv7l-hf-multiplatform.buildPackages.gcc;
              CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_RUNNER = "qemu-arm";
            };
          };
        }
      );
    };
}
