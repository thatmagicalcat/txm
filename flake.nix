{
  description = "txm -- Terminal TeX math rendering engine";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-26.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    { self, nixpkgs, rust-overlay }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      # Per-system pkgs with rust-overlay applied
      pkgsFor = system: import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };

      forAllSystems =
        f:
        builtins.listToAttrs (
          map (system: { name = system; value = f (pkgsFor system); }) supportedSystems
        );

      # Read version from Cargo.toml — single source of truth
      cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);

      # Shared arguments for buildRustPackage (used by all build strategies)
      txmArgs = {
        pname = cargoToml.package.name;
        version = cargoToml.package.version;
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;

        meta = {
          description = cargoToml.package.description or "Terminal math rendering engine with LaTeX support";
          homepage = cargoToml.package.repository or "https://github.com/thatmagicalcat/txm";
          license = [
            nixpkgs.lib.licenses.mit
            nixpkgs.lib.licenses.asl20
          ];
          mainProgram = "txm";
        };
      };

      # -- Native build (pure Nix, works on the building system) --
      mkTxm = pkgs: pkgs.rustPlatform.buildRustPackage txmArgs;

      # -- Cross-compile any target using cargo-zigbuild + rust-overlay --
      # rust-overlay provides the Rust toolchain with target stdlibs pre-built,
      # so the derivation is fully sandboxed (no rustup/network needed at build time).
      # Uses musl targets for Linux → fully static binaries, portable across distros.
      mkTxmCross =
        { rustTarget # e.g. "x86_64-unknown-linux-musl"
        , nixPlatform  # Nix system string, e.g. "x86_64-linux"
        }:
        let
          pkgsOverlay = pkgsFor "x86_64-linux";
          pkgsPlain = import nixpkgs { system = "x86_64-linux"; };

          # Rust toolchain with target stdlibs baked in (from overlay)
          rustWithTargets = pkgsOverlay.rust-bin.stable.latest.minimal.override {
            targets = [ rustTarget ];
          };

          # rustPlatform that uses the overlay's cargo/rustc (which has target stdlibs)
          # but with auditable disabled — Zig's linker doesn't support the
          # `--undefined` flag that nixpkgs' auditable feature injects.
          crossPlatform = pkgsPlain.makeRustPlatform {
            cargo = rustWithTargets;
            rustc = rustWithTargets;
          };

          # Detect Windows targets (produce .exe binaries)
          isWindows = pkgsPlain.lib.hasSuffix "windows-gnu" rustTarget;
          binName = if isWindows then "txm.exe" else "txm";
        in
        crossPlatform.buildRustPackage (txmArgs // {
          auditable = false;

          nativeBuildInputs = (txmArgs.nativeBuildInputs or [ ]) ++ [
            pkgsOverlay.cargo-zigbuild
            pkgsOverlay.zig
          ];

          buildPhase = ''
            runHook preBuild
            export HOME="$PWD"
            export ZIG_GLOBAL_CACHE_DIR="$TMPDIR/zig-cache"
            export CARGO_ZIGBUILD_CACHE_DIR="$TMPDIR/cargo-zigbuild-cache"
            cargo zigbuild --release --target ${rustTarget}
            runHook postBuild
          '';

          # Zig's Nix setup hook adds a checkPhase that runs `zig check`, but we
          # don't have a build.zig — zig is only used as a cross-linker.
          checkPhase = "true";

          installPhase = ''
            runHook preInstall
            mkdir -p $out/bin
            cp "target/${rustTarget}/release/${binName}" "$out/bin/${binName}"
            runHook postInstall
          '';

          meta = txmArgs.meta // {
            platforms = [ "x86_64-linux" nixPlatform ];
          };
        });
    in
    {
      # Per-system native packages (build for the system you're on)
      packages = forAllSystems (pkgs: {
        default = mkTxm pkgs;
      })
      # On x86_64-linux, also provide cross-compiled packages for all targets.
      # This lets a single Linux CI runner produce every platform binary.
      #
      # Linux targets use musl (fully static, portable across distros).
      //
        {
          x86_64-linux = let
            pkgs = pkgsFor "x86_64-linux";
          in {
            default = mkTxm pkgs;

            # Static Linux builds (musl, fully static, work on any Linux)
            x86_64-linux = mkTxmCross {
              rustTarget = "x86_64-unknown-linux-musl";
              nixPlatform = "x86_64-linux";
            };
            aarch64-linux = mkTxmCross {
              rustTarget = "aarch64-unknown-linux-musl";
              nixPlatform = "aarch64-linux";
            };

            # Cross-OS via cargo-zigbuild + rust-overlay (pure Nix, no rustup needed)
            x86_64-darwin = mkTxmCross {
              rustTarget = "x86_64-apple-darwin";
              nixPlatform = "x86_64-darwin";
            };
            aarch64-darwin = mkTxmCross {
              rustTarget = "aarch64-apple-darwin";
              nixPlatform = "aarch64-darwin";
            };
            x86_64-windows = mkTxmCross {
              rustTarget = "x86_64-pc-windows-gnu";
              nixPlatform = "x86_64-windows";
            };
          };
        };

      # Development shell with everything needed to build and cross-compile
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            # Rust toolchain with target stdlibs pre-installed (no rustup required)
            (rust-bin.stable.latest.default.override {
              targets = [
                "x86_64-apple-darwin"
                "aarch64-apple-darwin"
                "x86_64-pc-windows-gnu"
              ];
            })

            # Development tools
            clippy
            rustfmt
            rust-analyzer

            # Cross-compilation via Zig linker
            cargo-zigbuild
            zig
          ];

          shellHook = ''
            echo "txm devShell — tools available:"
            echo "  cargo build --release   (native build)"
            echo "  cargo clippy           (lints)"
            echo "  cargo fmt              (formatting)"
            echo "  cargo test             (tests)"
            echo "  cargo zigbuild         (cross-compile, see flake targets)"
            echo "  rust-analyzer          (LSP)"
          '';
        };
      });
    };
}
