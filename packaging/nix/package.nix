{
  lib,
  rustPlatform,
  makeWrapper,
  libX11,
  libXext,
  libXcursor,
  libXrandr,
  libXxf86vm,
  libXrender,
  libXtst,
  libXi,
  xrandr,
  libpulseaudio,
  libGL,
  glfw3-minecraft,
  openal,
  wayland,
  libxkbcommon,
}:
let
  loadDotenv = import ./loadDotenv.nix { inherit lib; };
  env = loadDotenv ../../build.env;
  # keep in sync with deps in flake.nix
  runtimeDeps = [
    libX11
    libXext
    libXcursor
    libXrandr
    libXxf86vm
    libXrender
    libXtst
    libXi
    xrandr
    libpulseaudio
    libGL
    glfw3-minecraft
    openal
    wayland
    libxkbcommon
  ];
in
rustPlatform.buildRustPackage {
  name = lib.pipe env.LAUNCHER_NAME [
    lib.toLower
    (lib.replaceStrings [ " " ] [ "-" ])
    (lib.replaceStrings [ "'" ] [ "" ])
  ];
  src = ./../..;
  # TODO: auto update this
  cargoHash = "sha256-K1fAjJHhr5PZ22VWW2x4Wg1E9MkTlVwSLCANHHh5Ve0=";

  USE_NATIVE_GLFW_DEFAULT = "true";
  nativeBuildInputs = [ makeWrapper ];
  postFixup = ''
    wrapProgram $out/bin/launcher \
      --set LD_LIBRARY_PATH "${lib.makeLibraryPath runtimeDeps}"
  '';

  meta = {
    description = env.LAUNCHER_DESCRIPTION or null;
    license = lib.licenses.mit;
    mainProgram = "launcher";
  };
}
