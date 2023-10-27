let
  pkgs = import <nixpkgs> {};
in
  pkgs.mkShell {
    buildInputs = with pkgs; [
        libusb1
    ];
    nativeBuildInputs = with pkgs; [
        pkg-config
    ];
  }
