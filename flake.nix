{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-23.11";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, naersk }:
    let
      system = "x86_64-linux";

      pkgs = import nixpkgs { inherit system; };
      lib = pkgs.lib;

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

        ffmpeg

        directx-headers

        vulkan-tools
        vulkan-headers
      ];

      examples = [
        "tutorial"
        "snow-shadertoy"
      ];

      defineExample = pname: naersk'.buildPackage {
        src = ./.;
        pname = pname;

        overrideMain = x: {
          preConfigure = ''
            cargo_build_options="$cargo_build_options --example $pname"
          '';
        };
      };

      examplePackages =
        lib.listToAttrs
          (
            lib.lists.forEach
              examples
              (pname:
                {
                  name = pname;
                  value = defineExample pname;
                }
              )
          ) //
        { default = defineExample "snow"; };

    in
    {
      formatter.${system} = pkgs.nixpkgs-fmt;

      packages.${system} = examplePackages;

      devShells.${system}.default = pkgs.mkShell rec {
        buildInputs = libs ++ tools;

        LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath libs}:/usr/lib/wsl/lib";
      };
    };
}
