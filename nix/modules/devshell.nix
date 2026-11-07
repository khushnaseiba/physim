{ inputs, ... }:
{
  perSystem =
    {
      config,
      self',
      pkgs,
      ...
    }:
    {
      devShells.default = pkgs.mkShell {
        name = "physim-shell";
        inputsFrom = [
          self'.devShells.rust
          config.pre-commit.devShell
        ];
        packages = with pkgs; [
          just
          nixd
          nixfmt
          bacon
          wasm-bindgen-cli
          wasm-pack
          binaryen
          nodejs
          python3
        ];
      };
    };
}
