{
  description = "Rikka Gecko: Gecko as an embeddable engine for Rikka";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          curl
          gh
          git
          jq
          rsync
          zstd
        ];
      };
    };
}
