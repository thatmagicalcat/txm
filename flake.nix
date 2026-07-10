{
  description = "txm -- Terminal TeX math rendering engine";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-26.05";
  };

  outputs =
    { self, nixpkgs }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forAllSystems =
        f:
        builtins.listToAttrs (
          map (system: {
            name = system;
            value = f (import nixpkgs { inherit system; });
          }) supportedSystems
        );
    in
    {
      packages = forAllSystems (pkgs: {
        default = pkgs.rustPlatform.buildRustPackage {
          pname = "txm";
          version = "0.1.1";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          meta = {
            description = "Terminal math rendering engine with LaTeX support";
            homepage = "https://github.com/thatmagicalcat/txm";
            license = with pkgs.lib.licenses; [
              mit
              asl20
            ];
            mainProgram = "txm";
          };
        };
      });

      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            clang
            clippy
            rust-analyzer
          ];
        };
      });
    };
}
