from py_ecc.fields import (
    bn128_FQ as FQ,
    bn128_FQ2 as FQ2,
    bn128_FQ12 as FQ12,
)
from py_ecc.bn128 import (
    bn128_curve as curve,
    bn128_pairing as pairing
)

import math

import json

import sys

def numberToArray(num, n, k):
    num = abs(num)
    # assume num >= 0
    registers = []
    for i in range(k):
        registers.append(str(int(num % (2**n))))
        num //= 2**n
    return registers

def Fpconvert(X, n, k):
    return numberToArray(X.n, n, k)

def Fp2convert(X, n, k):
    return [ numberToArray(X.coeffs[0].n, n, k) , numberToArray(X.coeffs[1].n, n, k) ]

n = 43
k = 6

with open(sys.argv[1], 'r') as proof_file:
    proof_data = proof_file.read()
proof = json.loads(proof_data)
x, y, z = tuple([FQ((int(x))) for x in proof["pi_a"]])
negpi_a = (x / z, - (y / z))
x, y, z = tuple([ FQ2([int(x[0]), int(x[1])]) for x in proof["pi_b"]])
pi_b = (x / z, y / z)
x, y, z = tuple([FQ((int(x))) for x in proof["pi_c"]])
pi_c = (x / z, y / z)
proofParameters = {
    "negpa": [Fpconvert(negpi_a[0], n, k), Fpconvert(negpi_a[1], n, k)],
    "pb": [ Fp2convert(pi_b[0], n, k), Fp2convert(pi_b[1], n, k)],
    "pc": [Fpconvert(pi_c[0], n, k), Fpconvert(pi_c[1], n, k)],
}

with open(sys.argv[2], 'r') as public_file:
    public_data = public_file.read()
pubInputs = json.loads(public_data)
pubParameters  = {
    "pubInput": [],
}
for pubInput in pubInputs:
    pubParameters["pubInput"].append(str(int(pubInput)))

proofParameters["pubInput"] = pubParameters["pubInput"]

with open('proof.json', 'w') as outfile:
    json.dump(proofParameters, outfile)
