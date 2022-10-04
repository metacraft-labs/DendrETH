pragma circom 2.0.3;

include "../../circuits/light_client_recursive.circom";

component main { public [ originator, nextHeaderHashNum ] } = LightClientRecursive(512);
