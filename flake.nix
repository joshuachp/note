{
  inputs = {
    nixpkgs = {
      url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk = {
      url = "github:nmattia/naersk/master";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils = {
      url = "github:numtide/flake-utils";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };
  outputs = {
    self,
    nixpkgs,
    flake-utils,
    naersk,
    fenix,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      naersk-lib = pkgs.callPackage naersk {};
    in rec {
      packages.default =
        (naersk-lib.override {
          inherit (fenix.packages.${system}.stable) cargo rustc;
        })
        .buildPackage {
          nativeBuildInputs = with pkgs; [installShellFiles];
          root = ./.;
          overrideMain = _: {
            postInstall = ''
              installShellCompletion --cmd note \
                --bash <($out/bin/note completion bash) \
                --fish <($out/bin/note completion fish) \
                --zsh <($out/bin/note completion zsh)
            '';
          };
        };

      app.default = flake-utils.lib.mkApp {
        drv = packages.default;
      };

      devShells.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          (fenix.packages.${system}.stable.withComponents [
            "cargo"
            "clippy"
            "rust-src"
            "rustc"
            "rustfmt"
          ])
          pre-commit
        ];
      };
    });
}
