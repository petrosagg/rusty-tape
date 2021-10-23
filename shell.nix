with import <nixpkgs> {};

stdenv.mkDerivation rec {
    name = "rusty-tape";
    buildInputs = [
        openssl
        pkgconfig
        youtube-dl
        freetype
        expat
        libxml2
        libxkbcommon
        ffmpeg
        rustup
        nodejs
        mpv
        vulkan-loader
        vulkan-tools
        xorg.libX11
        xorg.libXcursor
        xorg.libXrandr
        xorg.libXi
    ];
    shellHook = ''
        export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${lib.makeLibraryPath buildInputs}";
    '';
}
