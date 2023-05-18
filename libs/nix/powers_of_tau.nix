{
  lib,
  pkgs,
  ...
}: let
  inherit (pkgs) callPackage fetchurl runCommand writeTextFile;
  inherit (builtins) attrNames filter map readDir;
  inherit (lib) pipe hasSuffix removeSuffix;

  ptau_values = {
    power_8 = {
      url = "https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_08.ptau";
      hash = "sha256-90Hy3e4odZFcJNuKrpDQIfURgVM/HuO1i69ksELpFlQ=";
    };
    power_9 = {
      url = "https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_09.ptau";
      hash = "sha256-g21GmIXnfzYveMaiKF1Gg0prGoiY8OT+RWw6w2GYjFo=";
    };
    power_10 = {
      url = "https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_10.ptau";
      hash = "sha256-U9Dp0aNXZBKto5qCyP+t1/EQwbE95W2JtSpHLOXl7fQ=";
    };
    power_11 = {
      url = "https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_11.ptau";
      hash = "sha256-aIm5ZsnkckjjfHNG9taq2BmAvH7WmqnVXOnjrFrTf5Y=";
    };
    power_12 = {
      url = "https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_12.ptau";
      hash = "sha256-3PTqRzvxS5cc5fe3wdbOHEGo7QQs23W2XKkXjjo8fBc=";
    };
    power_13 = {
      url = "https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_13.ptau";
      hash = "sha256-lXUbUgfyCqgi8BEJkCMVwBwVJQMD/qzqK4qn3J/f7v0=";
    };
    power_14 = {
      url = "https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_14.ptau";
      hash = "sha256-SJvp5axl1ST3sWhbqsihg8bneST9tz0rgQXjNfJ3iV0=";
    };
    power_15 = {
      url = "https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_15.ptau";
      hash = "sha256-PvLsxbddaHBIzy1ZGVEZtC+wfFr2OcXyg9hL+mmCnn8=";
    };
    power_16 = {
      url = "https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_16.ptau";
      hash = "sha256-HEAau1fJzlMTcPMBXD51wIkuDzK4selKzg9mgtlpWSI=";
    };
    power_17 = {
      url = "https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_17.ptau";
      hash = "sha256-a2YqMkhnE5+xogoyTZC2/2GFbfsj9ZMmkJ8UsOJIOuA=";
    };
    power_18 = {
      url = "https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_18.ptau";
      hash = "sha256-6XDvp3dNqAEB4KwzbQg+8zOYVcmBElOTONcGsriaxpQ=";
    };
    power_19 = {
      url = "https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_19.ptau";
      hash = "sha256-P0KNGkB+RwTvkGlg4ACwMInl5uwpv2Wwe7Xj3gBfRwA=";
    };
    power_20 = {
      url = "https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_20.ptau";
      hash = "sha256-FZ0/k42UHgZ2fZnzC5/lmiRUAKSq4TjPjkEXMtei9s0=";
    };
    power_21 = {
      url = "https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_21.ptau";
      hash = "sha256-zcfJSmY1vJFGbYx9lvrv4dF+zJijWWp0jKHmyJX4wrQ=";
    };
    power_22 = {
      url = "https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_22.ptau";
      hash = "sha256-aKIb74cNXUqd45yPNevPBOGO+X4Uss0/TD45h2gh02I=";
    };
    power_23 = {
      url = "https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_23.ptau";
      hash = "sha256-BH8W112qzNb7P4WazIzCatH7Qe8DDaBwQx6V7bEm0Z0=";
    };
    power_24 = {
      url = "https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_24.ptau";
      hash = "sha256-AyZHq+En9FYvgRjdX4ZqtZXB8rvQPQqF/zc5pKln2b4=";
    };
    power_25 = {
      url = "https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_25.ptau";
      hash = "sha256-h+I2uE0S96RHF4f+FJI9sPaCMgaBasIqe2OYP877Fdw=";
    };
    power_26 = {
      url = "https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_26.ptau";
      hash = "sha256-UnxD+pEtLG2EiN5+sHw+PXVK3eVgGm8hXY7pIrOPJlU=";
    };
    power_27 = {
      url = "https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_27.ptau";
      hash = "sha256-ypwDTFWIm5rumj2QOpuNP8d3+9Op8R54DNrVWunxVHs=";
    };
    power_28 = {
      url = "https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final.ptau";
      hash = "sha256-Mr4Q2Z2HwQ+zm9hk4I246m8FsAO1xPOaLu3oJ5pDgYs=";
    };
  };

  fetchPtau = power: fetchurl ptau_values."power_${toString power}";

  all_ptau = lib.trivial.pipe (lib.range 8 28) [
    (map
      (x: {
        name = "ptau${toString x}";
        value = fetchPtau x;
      }))
    builtins.listToAttrs
  ];
  all_ptau_list = map (key: lib.getAttr key all_ptau) (attrNames all_ptau);

  ptau = writeTextFile {
    name = "all_ptau";
    text = ''
      ${toString (map (x: " ${x} ") all_ptau_list)};
    '';
  };
in
  all_ptau // {inherit ptau;}
