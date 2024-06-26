{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
    devshell.url = "github:numtide/devshell";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, utils, devshell, fenix, ... }@inputs:
    utils.lib.eachSystem [ "aarch64-linux" "i686-linux" "x86_64-linux" ]
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ devshell.overlays.default ];
          };
          rust-toolchain = with fenix.packages.${system};
            combine [
              stable.rustc
              stable.cargo
              stable.clippy
              latest.rustfmt
              targets.thumbv6m-none-eabi.stable.rust-std
            ];
        in
        rec {
          devShells.default = (pkgs.devshell.mkShell {
            imports = [ "${devshell}/extra/git/hooks.nix" ];
            name = "apex-rs-postcard-dev-shell";
            packages = with pkgs; [
              clang
              rust-toolchain
              rust-analyzer
              cargo-outdated
              cargo-audit
              cargo-udeps
              cargo-all-features
              cargo-watch
              nixpkgs-fmt
            ];
            git.hooks = {
              enable = true;
              pre-commit.text = "nix flake check";
            };
            commands = [
              { package = "git-cliff"; }
              { package = "treefmt"; }
              {
                name = "udeps";
                command = ''
                  PATH=${fenix.packages.${system}.latest.rustc}/bin:$PATH
                  cargo udeps $@
                '';
                help = pkgs.cargo-udeps.meta.description;
              }
              {
                name = "outdated";
                command = "cargo outdated";
                help = pkgs.cargo-outdated.meta.description;
              }
              {
                name = "audit";
                command = "cargo audit";
                help = pkgs.cargo-audit.meta.description;
              }
              {
                name = "verify-no_std";
                command = ''
                  cd $PRJ_ROOT
                  cargo build --target thumbv6m-none-eabi --no-default-features
                '';
                help =
                  "Verify that the library builds for no_std without std-features";
                category = "test";
              }
              {
                name = "verify-doc";
                command = ''
                  cd $PRJ_ROOT
                  cargo doc
                '';
                help =
                  "Verify that the documentation builds without problems";
                category = "test";
              }
              {
                name = "verify-features";
                command = ''
                  cd $PRJ_ROOT
                  cargo check-all-features $@
                '';
                help =
                  "Verify that all feature combinations build";
                category = "test";
              }
              {
                name = "verify-tests";
                command = ''
                  cd $PRJ_ROOT
                  cargo test-all-features $@
                '';
                help =
                  "Verify that all tests run for all feature combinations";
                category = "test";
              }
            ];
          });
          checks = {
            nixpkgs-fmt = pkgs.runCommand "nixpkgs-fmt"
              {
                nativeBuildInputs = [ pkgs.nixpkgs-fmt ];
              } "nixpkgs-fmt --check ${./.}; touch $out";
            cargo-fmt = pkgs.runCommand "cargo-fmt"
              {
                nativeBuildInputs = [ rust-toolchain ];
              } "cd ${./.}; cargo fmt --check; touch $out";
          };
        });
}

