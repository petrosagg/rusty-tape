with import <nixpkgs> {};

stdenv.mkDerivation {
    name = "rusty-tape";
    buildInputs = [
        openssl
        pkgconfig
        youtube-dl
        ffmpeg
        cargo
    ];
}
