{
  description                                       = "Jikyuu";

  inputs                                            = {
    nixpkgs.url                                     = "github:NixOS/nixpkgs/23.11";

    nixpkgs-unstable.url                            = "github:NixOS/nixpkgs/nixos-unstable";

    flake-utils.url                                 = "github:numtide/flake-utils";

    cargo2nix.url                                   = "github:cargo2nix/cargo2nix/release-0.11.0";
  };

  outputs                                           = {
    nixpkgs,
    flake-utils,
    cargo2nix,
    ...
  }:
    let
      systems                                       = [ "x86_64-linux" ];
    in (
      flake-utils.lib.eachSystem systems (system: (
        let
          pkgs                                      = import nixpkgs {
            inherit system;
            overlays                                = [cargo2nix.overlays.default];
          };
          pkgsRust                                  = pkgs.rustBuilder.makePackageSet {
            rustVersion                             = "1.75.0";
            packageFun                              = import ./Cargo.nix;
            workspaceSrc                            = ./.;
          };
          manifest                                  = (pkgs.lib.importTOML ./Cargo.toml).package;
          environment                               = {
            inherit system pkgs pkgsRust manifest;
          };
          name                                      = manifest.name;
        in rec {
          packages.${name}                          = (pkgsRust.workspace.${name} {});
          legacyPackages                            = packages;

          # `nix build`
          defaultPackage                            = packages.${name};

          # `nix run`
          apps.${name}                              = flake-utils.lib.mkApp {
            inherit name;
            drv                                     = packages.${name};
          };
          defaultApp                                = apps.${name};

          # `nix develop`
          devShells.${system}.default               = import ./shell.nix environment;
        }
      )
    )
  );
}


