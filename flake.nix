{
  description = "signal-orchestrate — generated ordinary Lock Signal contract";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-build = {
      url = "github:LiGoldragon/rust-build";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-build }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        rust = rust-build.lib.${system}.fromToolchainFile pkgs {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-gh/xTkxKHL4eiRXzWv8KP7vfjSk61Iq48x47BEDFgfk=";
        };
        inherit (rust) craneLib toolchain;
        ethosFilter = path: type: type == "regular" && pkgs.lib.hasSuffix ".ethos" path;
        src = rust.cleanSource {
          root = ./.;
          extraFilters = [ ethosFilter ];
        };
        commonArgs = { inherit src; strictDeps = true; nativeBuildInputs = [ pkgs.rustfmt ]; };
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      in
      {
        packages.default = craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; });
        checks = {
          build = craneLib.cargoBuild (commonArgs // { inherit cargoArtifacts; });
          test  = craneLib.cargoTest  (commonArgs // { inherit cargoArtifacts; });
          test-generated-contract = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
            cargoTestExtraArgs = "--test generated_contract";
          });
          test-datom-contract = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
            cargoTestExtraArgs = "--features datom --test generated_contract";
          });
          test-doc = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
            cargoTestExtraArgs = "--doc";
          });
          doc = craneLib.cargoDoc (commonArgs // {
            inherit cargoArtifacts;
            RUSTDOCFLAGS = "-D warnings";
          });
          fmt = craneLib.cargoFmt { inherit src; };
          no-free-functions = pkgs.runCommand "signal-orchestrate-no-free-functions" { inherit src; } ''
            ${builtins.readFile ./checks/no-free-functions.sh}
          '';
          no-inherent-methods = pkgs.runCommand "signal-orchestrate-no-inherent-methods" { inherit src; } ''
            ${builtins.readFile ./checks/no-inherent-methods.sh}
          '';
          clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          });
        };
        devShells.default = pkgs.mkShell {
          name = "signal-orchestrate";
          packages = [ pkgs.jujutsu pkgs.pkg-config toolchain ];
        };
      });
}
