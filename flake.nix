{
  description = "Note taking tool";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-24.11";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      crane,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        inherit (pkgs) mkShell lib;
        packages = self.packages.${system};
        toolchain = pkgs.rust-bin.stable.latest.default;
        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
        noteCargoToml = craneLib.crateNameFromCargoToml { cargoToml = ./note-cli/Cargo.toml; };
        # Only keeps markdown files
        fishFilter = path: _type: builtins.match ".*fish$" path != null;
        srcFilter = path: type: (fishFilter path type) || (craneLib.filterCargoSources path type);
      in
      {
        packages = {
          note = craneLib.buildPackage {
            pname = "note";
            version = noteCargoToml.version;
            src = lib.cleanSourceWith {
              src = craneLib.path ./.;
              filter = srcFilter;
              name = "source";
            };
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
          note = flake-utils.lib.mkApp { drv = packages.default; };
          default = self.apps.${system}.note;
        };
        devShells = {
          default = mkShell {
            # inputsFrom = [ packages.note ];
            packages = [
              pkgs.pre-commit

              pkgs.bazel_7
              pkgs.buildifier
            ];
          };
        };
      }
    );
}
