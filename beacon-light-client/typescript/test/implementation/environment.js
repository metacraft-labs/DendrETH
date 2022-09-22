const { digest } = require("@chainsafe/as-sha256");


// ===============================
//  / SMARTTS ENVIRONMENT SETUP \
// ===============================

const empty_bytes = (size) => Array(size).fill(0);

function Uint8ArrayToHexString(array) {
    let hex = "";
    for (let n of array) {
        hex = hex.concat(n.toString(16).padStart(2, "0"));
    }
    return "0x".concat(hex.padStart(64, "0"));
}

class TPair {
    constructor(first, second) {
        this.first = first;
        this.second = second;
    }

    fst() {
        return this.first;
    }

    snd() {
        return this.second;
    }
};

class TOption {
    constructor(o1, o2) {
        this.o1 = o1;
        this.o2 = o2;
    }

    openSome() {
        return new TPair(this.o1, this.o2);
    }
};

class SpClass {
    failWith(message) {
        throw new Error(message);
    }

    ediv(n1, n2) {
        return new TOption(Math.floor(n1 / n2), n1 % n2);
    }

    pack(n) {
        const array = [];
        const hexString = n.toString(16).padStart(Math.ceil(n.toString(16).length / 2) * 2, "0");

        for (let i = 0; i < hexString.length; i += 2)
            array.push(parseInt(hexString.slice(i, i + 2), 16));

        return array;
    }

    sha256(s) {
        return digest(s);
    }
};

const Sp = new SpClass();

module.exports = {
    Sp,
    empty_bytes,
    Uint8ArrayToHexString
};