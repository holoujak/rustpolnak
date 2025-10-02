{
  description = "Get devenv via `nix develop`";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          devenv
          # optional if we want direnv later
          # direnv
        ];
        # Automatically run devenv shell when entering nix develop
        shellHook = ''
          devenv shell
          exit
        '';
      };
    };
}
