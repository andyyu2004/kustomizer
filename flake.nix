{
  description = "Kustomizer development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils = {
      url = "github:numtide/flake-utils";
    };
    gotmpl = {
      url = "github:andyyu2004/gotmpl";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      gotmpl,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        kustomizer = pkgs.rustPlatform.buildRustPackage {
          pname = "kustomizer";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              # Weird issue with git dependencies
              # https://artemis.sh/2023/07/08/nix-rust-project-with-git-dependencies.html
              "serde_json-1.0.142" = "sha256-ToBU7gGWhZGvBqde6XSIyFC4bYynhe9Ql32mYAYQ/3s=";
            };
          };

          doCheck = false;

          meta = with pkgs.lib; {
            description = "Faster rust port of kustomize";
            homepage = "https://github.com/andyyu2004/kustomizer";
            license = licenses.mit;
          };
        };
      in
      {
        packages = {
          default = kustomizer;
          kustomizer = kustomizer;
        };

        formatter = pkgs.nixpkgs-fmt;

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Test dependencies
            kustomize
            kustomize-sops
            sops
            dyff
            gotmpl.packages.${system}.default
          ];

          shellHook = ''
            # kustomize-sops installs ksops binary at a custom path
            export PATH=${pkgs.kustomize-sops}/lib/viaduct.ai/v1/ksops:$PATH
          '';
        };
      }
    );
}
