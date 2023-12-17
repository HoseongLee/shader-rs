{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-23.11";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, naersk }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
      naersk' = pkgs.callPackage naersk { };

      libs = with pkgs; [
        libxkbcommon
        libGL

        xdg-desktop-portal-wlr

        wayland

        vulkan-loader
      ];

      tools = with pkgs; [
        rustc
        cargo
        rustfmt

        vulkan-tools
        vulkan-headers
      ];
    in
    {
      formatter.${system} = pkgs.nixpkgs-fmt;

      packages.${system}.default = naersk'.buildPackage {
        src = ./.;
      };

      devShells.${system}.default = pkgs.mkShell rec {
        buildInputs = libs ++ tools;

        LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath libs}";

        shellHook = ''
          zsh
        '';
      };
    };
}
