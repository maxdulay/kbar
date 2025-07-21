let pkgs = import <nixpkgs> { };
in pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    pkg-config
		pipewire.dev
    llvmPackages.libclang.lib
    rustPlatform.bindgenHook
  ];
  LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
}
