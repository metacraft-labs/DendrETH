pragma circom 2.0.3;

include "../../circuits/light_client.circom";

component main { public [ prevHeaderHashNum, nextHeaderHashNum, finalizedHeaderHashNum ] } = LightClient(512);
