{ inputs, ... }: {
  perSystem = { self', inputs', pkgs, system, ... }:
    let
      craneLib = inputs.crane.lib.${system};
      src = craneLib.cleanCargoSource (craneLib.path ./.);

      commonArgs = {
        inherit src;
        strictDeps = true;
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      };

      noogle-cli = craneLib.buildPackage commonArgs;

      checks = {
        inherit noogle-cli;
        cli-clippy = craneLib.cargoClippy (commonArgs // {
          cargoClippyExtraArgs = "--all-targets -- --deny warnings";
        });
        cli-fmt = craneLib.cargoFmt { inherit src; };
        cli-nextest = craneLib.cargoNextest (commonArgs // {
          partitions = 1;
          partitionType = "count";
        });
      };
    in
    {
      packages = { inherit noogle-cli; };
      inherit checks;
      devShells.cli = craneLib.devShell {
        # Inherit inputs from checks.
        inherit checks;
        inputsFrom = [ self'.devShells.default ];
      };
    };
}
