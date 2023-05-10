{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk = {
      url = "github:nmattia/naersk/master";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs =
    { self
    , nixpkgs
    , flake-utils
    , naersk
    , fenix
    }:
    flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs { inherit system; };
      toolchain = fenix.packages.${system}.stable.minimalToolchain;
      naersk' = pkgs.callPackages naersk {
        cargo = toolchain;
        rustc = toolchain;
      };
    in
    rec {
      packages = {
        note = naersk'.buildPackage {
          pname = "note";
          src = ./.;
          nativeBuildInputs = with pkgs; [ installShellFiles ];
          postInstall = ''
            installShellCompletion --cmd note \
              --bash <($out/bin/note completion bash) \
              --fish <($out/bin/note completion fish) \
              --zsh <($out/bin/note completion zsh)
          '';
        };
        default = packages.note;
      };
      apps = {
        note = flake-utils.lib.mkApp {
          drv = packages.default;
        };
        default = apps.note;
      };

      devShells = {
        default = pkgs.mkShell {
          inputsFrom = [
            packages.note
          ];
          packages = with pkgs; [
            (fenix.packages.${system}.stable.withComponents [
              "cargo"
              "clippy"
              "rust-src"
              "rustc"
              "rustfmt"
            ])
            rust-analyzer
            pre-commit
          ];
        };
      };
    });
}
