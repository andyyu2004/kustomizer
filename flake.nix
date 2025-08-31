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
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            kustomize
            kustomize-sops
            sops
            dyff
          ];

          shellHook = ''
            # kustomize-sops doesn't expose ksops binary by default for some reason
            export PATH=${pkgs.kustomize-sops}/lib/viaduct.ai/v1/ksops:$PATH
          '';
        };
      });
}
