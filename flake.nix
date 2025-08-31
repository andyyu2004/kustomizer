{
  description = "Kustomizer development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        kustomizer = pkgs.rustPlatform.buildRustPackage {
          pname = "kustomizer";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          buildInputs = [ ];
          nativeBuildInputs = [ ];

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

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Test dependencies
            kustomize
            kustomize-sops
            sops
            dyff
          ];

          shellHook = ''
            # kustomize-sops installs ksops binary at a custom path
            export PATH=${pkgs.kustomize-sops}/lib/viaduct.ai/v1/ksops:$PATH
          '';
        };
      });
}
