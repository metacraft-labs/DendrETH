pragma circom 2.1.5;

include "../../circuits/light_client_recursive.circom";

component main { public [ originator, nextHeaderHashNum ] } = LightClientRecursive(512);
