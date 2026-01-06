{
  perSystem = { pkgs, self', config, ... }: {
    devShells.default = pkgs.mkShell {
      packages = [
        pkgs.treefmt
        # Rust dependencies for CLI
        pkgs.cargo
        pkgs.rustc
        pkgs.rust-analyzer
        pkgs.clippy
        pkgs.rustfmt
        pkgs.pkg-config
      ];
      shellHook = ''
        ${self'.checks.pre-commit-check.shellHook}
      '';
    };
  };
}
