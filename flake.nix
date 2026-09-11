{
  description = "Standalone Nova top bar for Zellij";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    zjstatus = {
      url = "github:Yazelix/zjstatus/yazelix-tab-activity-pipe";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      fenix,
      zjstatus,
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      mkPkgs = system: nixpkgs.legacyPackages.${system};
      toolPackage =
        system: pkgs:
        let
          rustToolchain = fenix.packages.${system}.combine [
            fenix.packages.${system}.stable.cargo
            fenix.packages.${system}.stable.rustc
          ];
          rustPlatform = pkgs.makeRustPlatform {
            cargo = rustToolchain;
            rustc = rustToolchain;
          };
          source = pkgs.lib.cleanSourceWith {
            name = "nova-bar-source";
            src = ./.;
            filter =
              path: _type:
              let
                relativePath = pkgs.lib.removePrefix ((toString ./.) + "/") (toString path);
              in
              relativePath != "target"
              && !pkgs.lib.hasPrefix "target/" relativePath
              && relativePath != ".git"
              && !pkgs.lib.hasPrefix ".git/" relativePath;
          };
        in
        rustPlatform.buildRustPackage {
          pname = "nova_bar_tools";
          version = "0.1.0";

          src = source;
          cargoLock.lockFile = ./Cargo.lock;

          meta = {
            description = "Nova Bar widget command";
            homepage = "https://github.com/Yazelix/nova-bar";
            license = pkgs.lib.licenses.asl20;
            mainProgram = "nova_bar_widget";
          };
        };
      barPackage =
        system: pkgs:
        let
          tools = toolPackage system pkgs;
          zjstatusPackage = zjstatus.packages.${system}.default;
        in
        pkgs.stdenvNoCC.mkDerivation {
          pname = "nova_bar";
          version = "0.1.0";
          src = pkgs.lib.cleanSourceWith {
            name = "nova-bar-assets";
            src = ./.;
            filter =
              path: _type:
              let
                relativePath = pkgs.lib.removePrefix ((toString ./.) + "/") (toString path);
              in
              relativePath == "presets"
              || relativePath == "presets/examples"
              || pkgs.lib.hasPrefix "presets/" relativePath
              || relativePath == "README.md";
          };

          dontConfigure = true;
          dontBuild = true;

          installPhase = ''
            runHook preInstall

            install -Dm644 ${zjstatusPackage}/bin/zjstatus.wasm "$out/share/nova_bar/zjstatus.wasm"
            substitute presets/nova_bar.kdl "$out/share/nova_bar/nova_bar.kdl" \
              --replace-fail "__NOVA_BAR_ZJSTATUS_WASM__" "file:$out/share/nova_bar/zjstatus.wasm"
            install -Dm644 presets/nova_bar.kdl "$out/share/nova_bar/nova_bar.template.kdl"
            install -Dm644 presets/nova_runtime_bar.template.kdl "$out/share/nova_bar/nova_runtime_bar.template.kdl"
            install -Dm755 ${tools}/bin/nova_bar_widget "$out/bin/nova_bar_widget"
            cp -R presets/examples "$out/share/nova_bar/examples"
            install -Dm644 README.md "$out/share/doc/nova_bar/README.md"

            runHook postInstall
          '';

          doInstallCheck = true;
          nativeInstallCheckInputs = [
            pkgs.coreutils
            pkgs.gnugrep
          ];
          installCheckPhase = ''
            runHook preInstallCheck

            test -s "$out/share/nova_bar/zjstatus.wasm"
            test -x "$out/bin/nova_bar_widget"
            grep -q 'location="file:' "$out/share/nova_bar/nova_bar.kdl"
            ! grep -q '__NOVA_BAR_ZJSTATUS_WASM__' "$out/share/nova_bar/nova_bar.kdl"
            test -s "$out/share/nova_bar/nova_runtime_bar.template.kdl"
            test -s "$out/share/nova_bar/examples/custom_command_widgets.kdl"
            test -s "$out/share/nova_bar/examples/standalone_zellij_layout.kdl"
            test -s "$out/share/nova_bar/examples/nova_runtime_widgets.kdl"

            runHook postInstallCheck
          '';

          passthru = {
            presetPath = "share/nova_bar/nova_bar.kdl";
            templatePath = "share/nova_bar/nova_bar.template.kdl";
            runtimeTemplatePath = "share/nova_bar/nova_runtime_bar.template.kdl";
            examplesPath = "share/nova_bar/examples";
            widgetPath = "bin/nova_bar_widget";
            wasmPath = "share/nova_bar/zjstatus.wasm";
          };

          meta = {
            description = "Standalone Nova top bar for Zellij";
            homepage = "https://github.com/Yazelix/nova-bar";
            license = pkgs.lib.licenses.asl20;
            mainProgram = "nova_bar_widget";
          };
        };
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = mkPkgs system;
          bar = barPackage system pkgs;
          tools = toolPackage system pkgs;
        in
        {
          default = bar;
          nova_bar = bar;
          nova_bar_widget = tools;
        }
      );

      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.nova_bar_widget}/bin/nova_bar_widget";
        };
        nova_bar_widget = {
          type = "app";
          program = "${self.packages.${system}.nova_bar_widget}/bin/nova_bar_widget";
        };
      });

      checks = forAllSystems (system: {
        nova_bar = self.packages.${system}.nova_bar;
        nova_bar_widget = self.packages.${system}.nova_bar_widget;
      });
    };
}
