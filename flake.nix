{
  description = "";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable-small";
    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";

    flake-utils.url = "github:numtide/flake-utils";
    systems.url = "github:nix-systems/default";
    flake-utils.inputs.systems.follows = "systems";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.flake-utils.follows = "flake-utils";

    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    crane.inputs.flake-utils.follows = "flake-utils";
    crane.inputs.rust-overlay.follows = "rust-overlay";
  };

  nixConfig.extra-substituters = [
    "https://cache.garnix.io"
  ];
  nixConfig.extra-trusted-public-keys = [
    "cache.garnix.io:CTFPyKSLcx5RMJKfLo5EEPUObbA78b0YQ2DTCJXqr9g="
  ];

  outputs = inputs @ { flake-parts, self, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        # Possible MacOS support
        # "aarch64-darwin"
        # "x86_64-darwin"
        "x86_64-linux"
      ];
      perSystem = { config, self', pkgs, system, lib, ... }: rec {
        packages = {
          kytan = inputs.crane.lib.${system}.buildPackage {
            name = "kytan";
            src = self;

            # Only use flake for build not test
            doCheck = false;
            meta = with lib; {
              description = "kytan";
              homepage = "https://github.com/changlan/kytan";
              license = licenses.asl20;
              platforms = platforms.unix;
            };
          };
          default = self'.packages.kytan;
        };
      };
    };
}
