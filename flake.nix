{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    fenix = {
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };

  };

  outputs =
    {
      self,
      flake-utils,
      fenix,
      naersk,
      nixpkgs,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };
        toolchain =
          with fenix.packages.${system};
          combine [
            complete.rustc
            complete.cargo
            complete.rust-src
            complete.rust-analyzer
            complete.clippy
          ];

        naersk' = pkgs.callPackage naersk {
          cargo = toolchain;
          rustc = toolchain;
        };

      in
      rec {
        # For `nix build` & `nix run`:
        defaultPackage = naersk'.buildPackage {
          src = ./.;
        };

        # For `nix develop` (optional, can be skipped):
        devShell = pkgs.mkShell {
          nativeBuildInputs = [
            toolchain
          ];
        };

        nixosModules.default =
          {
            lib,
            pkgs,
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

            group =
              entries: attr:
              lib.mapAttrs (_: xs: map (x: x.${attr}) xs) (lib.groupBy (x: x.persistentStoragePath) entries);

            out_conf = {
              directories = group allPersistentStoragePaths.directories "dirPath";
              files = group allPersistentStoragePaths.files "filePath";
            };
          in
          {
            environment.etc."leaves.json".text = builtins.toJSON out_conf;
          };
      }
    );
}
