{ self, ... }:
{
  perSystem =
    {
      pkgs,
      self',
      ...
    }:
    {
      checks = {
        # Formatting check for rust files
        rust-fmt = pkgs.stdenvNoCC.mkDerivation {
          name = "rust-format-check";
          src = self;
          nativeBuildInputs = [
            pkgs.cargo
            pkgs.findutils
            pkgs.rustfmt
          ];
          buildPhase = ''
            cd "$src"
            cargo fmt --check
          '';
          installPhase = ''
            mkdir -p $out
          '';
        };
        # test check
        cargo-test = pkgs.stdenvNoCC.mkDerivation {
          name = "cargo-test-check";
          src = self;
          nativeBuildInputs = [
            pkgs.cargo
            pkgs.findutils
          ];
          buildPhase = ''
            cargo test
          '';
          installPhase = ''
            mkdir -p $out
          '';
        };
      };
    };
}
