import type * as T from '../types/basic-types';

import * as C from '../utils/Constants.contract';

// ===========
//  / UTILS \
// ===========

@Contract
export class Helpers extends C.Constants {
    pow = (base: T.Uint64, exponent: T.Uint64): T.Uint64 => {
        if (base == 1) {
            return base;
        }

        let result: T.Uint64 = 1;
        for (let i = 0; i < exponent; i += 1) {
            result = result * base;
        }

        return result;
    };

    getElementInUintArrayAt = (index: T.Uint64, arr: TList<T.Uint64>): T.Uint64 => {
        if (index >= (arr as TList<T.Uint64>).size()) {
            return 0 as T.Uint64;
        }

        let i: T.Uint64 = 0;
        for (const ele of arr as TList<T.Uint64>) {
            if (i == index) {
                return ele;
            }
            i += 1;
        }

        Sp.failWith('Helpers: Invalid params!');
        return 0 as T.Uint64;
    };

    getElementInBytesArrayAt = (index: T.Uint64, arr: TList<T.Bytes32>): T.Bytes32 => {
        if (index >= (arr as TList<T.Bytes32>).size()) {
            return '0x0000000000000000000000000000000000000000000000000000000000000000' as T.Bytes32;
        }

        let i = 0;
        for (const ele of arr as TList<T.Bytes32>) {
            if (i == index) {
                return ele;
            }
            i += 1;
        }

        Sp.failWith('Helpers: Invalid params!');
        return '0x0000000000000000000000000000000000000000000000000000000000000000' as T.Bytes32;
    };

    setElementInBytesArrayAt = (index: T.Uint64, arr: TList<T.Bytes32>, element: T.Bytes): TList<T.Bytes32> => {
        if (index >= (arr as TList<T.Bytes32>).size()) {
            Sp.failWith('Helpers: Invalid params!');
        }

        let i = 0;
        const result_array: TList<T.Bytes32> = [];
        for (const e of arr as TList<T.Bytes32>) {
            if (i != index) {
                result_array.push(e);
            } else {
                result_array.push(element);
            }
            i += 1;
        }

        return result_array.reverse();
    };
}

Dev.compileContract('compilation', new Helpers());
