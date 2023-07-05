{
  pkgs,
  crane,
  rustToolChain,
  advisory-db,
}: let
  craneLib = crane.overrideToolchain rustToolChain;
  src = craneLib.cleanCargoSource (craneLib.path ./..);

  commonArgs = {
    inherit src;

    pname = "dummyhttp";
    version = "1.0.3";

    nativeBuildInputs = with pkgs; [
      cmake
      fontconfig
      openssl
      pkgconfig
    ];
    doCheck = false;
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;

  dummyhttpCrate = craneLib.buildPackage (commonArgs
    // {
      inherit cargoArtifacts;
    });

  dummyhttpDoc = craneLib.cargoDoc (commonArgs
    // {
      inherit cargoArtifacts;
    });
in {
  default = dummyhttpCrate;

  doc = dummyhttpDoc;

  container = pkgs.dockerTools.buildImage {
    name = "dummyhttp";
    tag = "latest";

    fromImageName = "nginx";
    fromImageTag = "latest";

    copyToRoot = pkgs.buildEnv {
      name = "image-root";
      paths = [dummyhttpCrate];
      pathsToLink = ["/bin"];
    };

    # runAsRoot = ''
    #   #!${pkgs.runtimeShell}
    #   mkdir -p /data
    # '';

    config = {
      Cmd = ["/bin/dummyhttp"];
      # WorkingDir = "/data";
      # Volumes = {"/data" = {};};
    };

    diskSize = 1024;
    buildVMMemorySize = 512;
  };

  checks = {
    inherit dummyhttpCrate;

    # Format.
    fmt = craneLib.cargoFmt {
      inherit src;
    };

    # Clippy.
    clippy = craneLib.cargoClippy (commonArgs
      // {
        inherit cargoArtifacts;
        cargoClippyExtraArgs = "--all-targets -- --deny warnings";
      });

    # Test.
    test = craneLib.cargoTest (commonArgs
      // {
        inherit cargoArtifacts;
        doCheck = true;
      });

    # Audit.
    audit = craneLib.cargoAudit {
      inherit src advisory-db;
    };
  };
}
