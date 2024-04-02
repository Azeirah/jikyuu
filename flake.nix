{
  description                                       = "Jikyuu";

  inputs                                            = {
    nixpkgs.url                                     = "github:NixOS/nixpkgs/23.11";

    systems.url                                     = "path:./flake.systems.nix";
    systems.flake                                   = false;

    flake-utils.url                                 = "github:numtide/flake-utils";
    flake-utils.inputs.systems.follows              = "systems";

    fenix.url                                       = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows                    = "nixpkgs";

    # task-runner.url                                 = "gitlab:ox_os/task-runner";
    # task-documentation.url                          = "gitlab:ox_os/task-documentation";
  };

  outputs                                           = {
    nixpkgs,
    flake-utils,
    # task-runner,
    # task-documentation,
    fenix,
    ...
  }@inputs:
    let
      mkPkgs                                        =
        system:
          pkgs: (
            # NixPkgs
            import pkgs { inherit system; }
            //
            # Custom Packages.
            {
              # task-documentation                    = task-documentation.defaultPackage."${system}";
            }
          );

    in (
      flake-utils.lib.eachDefaultSystem (system: (
        let
          pkgs                                      = mkPkgs system nixpkgs;
          manifest                                  = (pkgs.lib.importTOML ./Cargo.toml).package;
          environment                               = {
            inherit pkgs;
            inherit manifest;
            toolchain                               = fenix.packages.${system}.minimal.toolchain;
          };
          name                                      = manifest.name;
        in rec {
          packages.${name}                          = pkgs.callPackage ./default.nix environment;
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
          devShells.default                         = import ./shell/default.nix {
            inherit mkPkgs system environment;
            flake-inputs                            = inputs;
          };
        }
      )
    )
  );
}
