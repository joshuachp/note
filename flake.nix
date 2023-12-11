{
  description = "Note taking tool";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs =
    { self
    , nixpkgs
    , flake-utils
    , crane
    , rust-overlay
    , ...
    }:
    flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
      };
      inherit (pkgs) mkShell;
      packages = self.packages.${system};
      toolchain = pkgs.rust-bin.nightly.latest.default;
      craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
      noteCargoToml = craneLib.crateNameFromCargoToml {
        cargoToml = ./note-cli/Cargo.toml;
      };
    in
    {
      packages = {
        note = craneLib.buildPackage {
          pname = "note";
          version = noteCargoToml.version;
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          nativeBuildInputs = with pkgs; [ installShellFiles ];
          postInstall = ''
            installShellCompletion --cmd note \
              --bash <($out/bin/note completion bash) \
              --fish <($out/bin/note completion fish) \
              --zsh <($out/bin/note completion zsh)
          '';
          cargoExtraArgs = "--package=note-cli";
        };
        default = packages.note;
      };
      apps = {
        note = flake-utils.lib.mkApp {
          drv = packages.default;
        };
        default = self.apps.${system}.note;
      };
      devShells = {
        default = mkShell {
          inputsFrom = [
            packages.note
          ];
          packages = with pkgs; [
            rust-analyzer
            pre-commit
          ];
        };
      };
    });
}
