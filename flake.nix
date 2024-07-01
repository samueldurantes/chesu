{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/release-23.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [(import rust-overlay)];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
      inherit (pkgs) lib;
      inherit (pkgs.stdenv) isDarwin;
    in {
      devShells.default = pkgs.mkShell {
        name = "chesu";
        shellHook = ''
            export LIBCLANG_PATH="${pkgs.llvmPackages_17.libclang.lib}/lib"
        '';
        packages = let
          commonPackages = with pkgs; [
            (rust-bin.stable."1.76.0".default.override {
              extensions = ["rust-src" "rust-analyzer" "llvm-tools"];
            })
            (writeShellApplication {
              name = "cargo-nightly-fmt";
              runtimeInputs = [
                (rust-bin.selectLatestNightlyWith (toolchain:
                  toolchain.minimal.override {
                    extensions = ["rustfmt"];
                  }))
              ];
              text = ''cargo fmt "$@"'';
            })
            cargo-nextest
            cargo-llvm-cov

            cmake
            postgresql_15

            openssl
            boringssl

            pkg-config
            llvmPackages_17.llvm
            llvmPackages_17.bintools
            llvmPackages_17.clang
            llvmPackages_17.libclang
            libiconv
          ];
          darwinPackages = lib.optionals isDarwin (with pkgs.darwin.apple_sdk.frameworks; [
            CoreFoundation
            CoreServices
            SystemConfiguration
          ]);
        in (commonPackages ++ darwinPackages);
      };
    });
}
