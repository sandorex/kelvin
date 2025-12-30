{
  description = "Your friendly temperature monitor";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      inherit self;
      system = "x86_64-linux";

      pkgs = import nixpkgs { inherit system; };

      cargoConfig = (builtins.fromTOML ./Cargo.toml);

      # for vergen_git2
      VERGEN_IDEMPOTENT = "1";
      VERGEN_GIT_SHA = if (self ? "rev") then (builtins.substring 0 7 self.rev) else "nix-dirty";
    in
    rec {
      packages.${system}.default = pkgs.rustPlatform.buildRustPackage rec {
        pname = cargoConfig.package.name;
        version = cargoConfig.package.version;

        src = ./.;
        cargoLock = {
          lockFile = ./Cargo.lock;
        };

        inherit VERGEN_IDEMPOTENT VERGEN_GIT_SHA;
      };

      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          git
          cargo
          rustc
          rust-analyzer # lsp
        ];

        inherit VERGEN_IDEMPOTENT VERGEN_GIT_SHA;
      };
    };
}
