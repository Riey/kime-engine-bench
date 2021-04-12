let
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs {};
in
pkgs.mkShell {
  name = "kime-engine-bench";
  buildInputs = with pkgs; [
    libhangul
  ];
  nativeBuildInputs = with pkgs; [
    rustc
    cargo
    rust-bindgen
  ];
}

