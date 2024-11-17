# Modified from https://github.com/NixOS/nixpkgs/blob/e0464e47880a69896f0fb1810f00e0de469f770a/pkgs/applications/virtualization/sail-riscv/default.nix

{ stdenv
, fetchFromGitHub
, fetchpatch
, lib
, arch
, pkgs
, ocamlPackages
, ocaml
, zlib
, z3
}:


stdenv.mkDerivation rec {
  pname = "sail-riscv";
  version = "8a2e0f565808e6f1ee02c6369005cd153b8205a0";

  src = fetchFromGitHub {
    owner = "riscv";
    repo = pname;
    rev = version;
    hash = "sha256-0dI/M6u/wSa41Xg0uWoJIN4BkAZ//WXommknA+JFPK4=";
  };

  nativeBuildInputs = with ocamlPackages; [ ocamlbuild findlib ocaml z3 pkgs.pkg-config
    (sail.overrideAttrs (previousAttrs: {
      version = "0.18";

      src = fetchFromGitHub {
        owner = "rems-project";
        repo = "sail";
        rev = "0.18";
        hash = "sha256-QvVK7KeAvJ/RfJXXYo6xEGEk5iOmVsZbvzW28MHRFic=";
      };

      propagatedBuildInputs = previousAttrs.propagatedBuildInputs ++ [ ocamlPackages.menhirLib ];
    }))];
  buildInputs = with ocamlPackages; [ zlib linksem ];
  strictDeps = true;

  postPatch = ''
    rm -r prover_snapshots
  '' + lib.optionalString stdenv.hostPlatform.isDarwin ''
    substituteInPlace Makefile --replace "-flto" ""
  '';

  makeFlags = [
    "SAIL=sail"
    "ARCH=${arch}"
    "SAIL_DIR=${ocamlPackages.sail}/share/sail"
    "LEM_DIR=${ocamlPackages.sail}/share/lem"
  ];

  installPhase = ''
    sail -version
    runHook preInstall

    mkdir -p $out/bin
    cp c_emulator/riscv_sim_${arch} $out/bin
    #mkdir $out/share/
    #cp -r generated_definitions/{coq,hol4,isabelle} $out/share/

    runHook postInstall
  '';


  meta = with lib; {
    homepage = "https://github.com/riscv/sail-riscv";
    description = "Formal specification of the RISC-V architecture, written in Sail";
    # maintainers = with maintainers; [ genericnerdyusername ];
    broken = stdenv.hostPlatform.isDarwin && stdenv.hostPlatform.isAarch64;
    license = licenses.bsd2;
  };
}
