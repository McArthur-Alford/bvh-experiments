{
  inputs = {
    crane = {
      url = "github:ipetkov/crane";
      inputs = {
        flake-utils.follows = "flake-utils";
        nixpkgs.follows = "nixpkgs";
      };
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };

  outputs =
    {
      self,
      crane,
      fenix,
      flake-utils,
      nixpkgs,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
        };

        rust-pkgs = fenix.packages.${system}.stable;
        craneLib = (crane.mkLib pkgs).overrideToolchain (rust-pkgs.toolchain);

        runtimeDeps = (
          with pkgs;
          [
            pkg-config
            libxkbcommon
            alsa-lib
            udev
            wayland
            vulkan-loader
          ]
          ++ (with xorg; [
            libXcursor
            libXrandr
            libXi
            libX11
          ])
        );
      in
      {
        packages.default = craneLib.buildPackage {
          # src = craneLib.cleanCargoSource ./.;
          # pname = "wgpu-runner";
          # cargoExtraArgs = "-p wgpu-runner";

          src = craneLib.cleanCargoSource ./.;
        };

        devShells.default = craneLib.devShell {
          # RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath runtimeDeps}";

          packages =
            (with pkgs; [
              (rust-pkgs.withComponents [
                "cargo"
                "clippy"
                "rust-src"
                "rustc"
                "rustfmt"
              ])
              rust-analyzer
              just
            ])
            ++ runtimeDeps;
        };
      }
    );
}
