{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.fenix.follows = "fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    fenix = {
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };

  };

  outputs =
    {
      flake-utils,
      fenix,
      naersk,
      nixpkgs,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };
        withComponents = fenix.packages.${system}.complete.withComponents;
        baseComponents = [
          "rustc"
          "cargo"
          "clippy"
        ];
        toolchain = withComponents baseComponents;
        devToolchain = withComponents (
          baseComponents
          ++ [
            "rust-src"
            "rust-analyzer"
          ]
        );

        naersk' = pkgs.callPackage naersk {
          cargo = toolchain;
          rustc = toolchain;
        };

      in
      rec {
        # For `nix build` & `nix run`:
        packages.default = naersk'.buildPackage {
          src = ./.;
        };

        # For `nix develop` (optional, can be skipped):
        shells.default = pkgs.mkShell {
          nativeBuildInputs = [
            devToolchain
          ];
        };

        homeManagerModules.default =
          {
            lib,
            config,
            ...

          }:
          let
            cfg = config.programs.leaves;
            toml = pkgs.formats.toml { };
          in
          {

            options.programs.leaves = {
              enable = lib.mkEnableOption "leaves";
              package = lib.mkOption {
                type = lib.types.package;
                default = packages.default;
                description = "The leaves package to use.";
              };

              settings = {
                old_roots_path = lib.mkOption {
                  default = "/persistent/old_roots";
                  example = "/persistent/roots";
                  description = "Path to old_roots volume.";
                  type = lib.types.str;
                };
                exclude_current = lib.mkOption {
                  default = null;
                  example = "current";
                  description = "Name of link to current root.";
                  type = lib.types.nullOr lib.types.str;
                };
              };
            };

            config = lib.mkIf cfg.enable {
              home.packages = [ cfg.package ];
              xdg.configFile."leaves/leaves.toml".source = toml.generate "leaves.toml" (
                lib.filterAttrs (_: v: v != null) cfg.settings
              );
            };
          };

        nixosModules.default =
          {
            lib,
            config,
            ...
          }:
          let
            cfg = config.environment.persistence;
            inherit (lib)
              flatten
              catAttrs
              attrValues
              filter
              mapAttrsToList
              zipAttrsWith
              ;

            allPersistentStoragePaths =
              let
                # All enabled system paths
                nixos = filter (v: v.enable) (attrValues cfg);

                # Get the files and directories from the `users` submodules of
                # enabled system paths
                nixosUsers = flatten (map attrValues (catAttrs "users" nixos));

                # Fetch enabled paths from all Home Manager users who have the
                # persistence module loaded
                homeManager =
                  let
                    paths = flatten (
                      mapAttrsToList (_name: value: attrValues (value.home.persistence or { }))
                        config.home-manager.users or { }
                    );
                  in
                  filter (v: v.enable) paths;
              in
              zipAttrsWith (_: flatten) (nixos ++ nixosUsers ++ homeManager);

            allExtraPaths =
              let
                one = cfg: if cfg ? atlas then cfg.atlas.extraPaths else [ ];
                nixos = one config;
                homeManager = flatten (mapAttrsToList (_: one) config.home-manager.users or { });
              in
              nixos ++ homeManager;

            group =
              entries: attr:
              lib.mapAttrs (_: xs: map (x: x.${attr}) xs) (lib.groupBy (x: x.persistentStoragePath) entries);

            out_conf = {
              directories = group allPersistentStoragePaths.directories "dirPath";
              files = group allPersistentStoragePaths.files "filePath";
              extra = allExtraPaths;
            };
          in
          {
            environment.etc."leaves.json".text = builtins.toJSON out_conf;
            home-manager.sharedModules = [ homeManagerModules.default ];
          };
      }
    );
}
