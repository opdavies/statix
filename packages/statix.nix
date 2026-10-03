{
  rustPlatform,
  lib,
  clippy,
  gitMinimal,
}:
rustPlatform.buildRustPackage {
  RUSTFLAGS = "-D warnings";
  cargoLock.lockFile = ../Cargo.lock;

  checkPhase = ''
    runHook preCheck

    cargo clippy --all-targets --all-features
    cargoCheckHook

    runHook postCheck
  '';

  meta = {
    description = "Lints and suggestions for the Nix programming language";
    homepage = "https://github.com/molybdenumsoftware/statix";
    license = lib.licenses.mit;
    mainProgram = "statix";
  };

  nativeBuildInputs = [ clippy ];

  nativeCheckInputs = [ gitMinimal ];

  pname = "statix";

  src = lib.fileset.toSource {
    fileset = lib.fileset.unions [
      (lib.fileset.fileFilter (
        file:
        lib.any lib.id [
          (file.hasExt "rs")
          (file.hasExt "snap")
          (file.name == "Cargo.toml")
        ]
      ) ../.)
      ../Cargo.lock
      ../insta.yaml
    ];

    root = ../.;
  };

  version = "0.6.0-git";
}
