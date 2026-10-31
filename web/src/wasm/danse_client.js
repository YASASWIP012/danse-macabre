/* @ts-self-types="./danse_client.d.ts" */

/**
 * @param {bigint} game
 * @param {number} index
 * @param {string} secret
 * @param {string} payout_key
 * @returns {string}
 */
export function ante_proof(game, index, secret, payout_key) {
    let deferred5_0;
    let deferred5_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16) >>> 0;
        const val3 = BigInt.asIntN(64, game);
        const val4 = +index;
        let ptr0, len0, ptr1, len1;
        try {
            ptr0 = passStringToWasm0(secret, wasm.__wbindgen_export3, wasm.__wbindgen_export4);
            len0 = WASM_VECTOR_LEN;
            ptr1 = passStringToWasm0(payout_key, wasm.__wbindgen_export3, wasm.__wbindgen_export4);
            len1 = WASM_VECTOR_LEN;
        } catch (e) {
            if (ptr1) { wasm.__wbindgen_export5(ptr1, len1 * 1, 1); }
            if (ptr0) { wasm.__wbindgen_export5(ptr0, len0 * 1, 1); }
            throw e;
        }
        wasm.ante_proof(retptr, val3, val4, ptr0, len0, ptr1, len1);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred5_0 = r0;
        deferred5_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export5(deferred5_0, deferred5_1, 1);
    }
}

/**
 * @param {bigint} game
 * @param {number} index
 * @param {string} secret
 * @param {number} nonce
 * @param {string} sealer
 * @returns {string}
 */
export function fee_proof(game, index, secret, nonce, sealer) {
    let deferred6_0;
    let deferred6_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16) >>> 0;
        const val3 = BigInt.asIntN(64, game);
        const val4 = +index;
        const val5 = +nonce;
        let ptr0, len0, ptr1, len1;
        try {
            ptr0 = passStringToWasm0(secret, wasm.__wbindgen_export3, wasm.__wbindgen_export4);
            len0 = WASM_VECTOR_LEN;
            ptr1 = passStringToWasm0(sealer, wasm.__wbindgen_export3, wasm.__wbindgen_export4);
            len1 = WASM_VECTOR_LEN;
        } catch (e) {
            if (ptr1) { wasm.__wbindgen_export5(ptr1, len1 * 1, 1); }
            if (ptr0) { wasm.__wbindgen_export5(ptr0, len0 * 1, 1); }
            throw e;
        }
        wasm.fee_proof(retptr, val3, val4, ptr0, len0, val5, ptr1, len1);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred6_0 = r0;
        deferred6_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export5(deferred6_0, deferred6_1, 1);
    }
}

/**
 * @param {bigint} game
 * @param {string} ticket
 * @param {number} index
 * @param {string} key
 * @returns {string}
 */
export function invite_proof(game, ticket, index, key) {
    let deferred5_0;
    let deferred5_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16) >>> 0;
        const val3 = BigInt.asIntN(64, game);
        const val4 = +index;
        let ptr0, len0, ptr1, len1;
        try {
            ptr0 = passStringToWasm0(ticket, wasm.__wbindgen_export3, wasm.__wbindgen_export4);
            len0 = WASM_VECTOR_LEN;
            ptr1 = passStringToWasm0(key, wasm.__wbindgen_export3, wasm.__wbindgen_export4);
            len1 = WASM_VECTOR_LEN;
        } catch (e) {
            if (ptr1) { wasm.__wbindgen_export5(ptr1, len1 * 1, 1); }
            if (ptr0) { wasm.__wbindgen_export5(ptr0, len0 * 1, 1); }
            throw e;
        }
        wasm.invite_proof(retptr, val3, ptr0, len0, val4, ptr1, len1);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred5_0 = r0;
        deferred5_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export5(deferred5_0, deferred5_1, 1);
    }
}

/**
 * @param {bigint} game
 * @param {number} index
 * @param {string} secret
 * @returns {string}
 */
export function join_proof(game, index, secret) {
    let deferred4_0;
    let deferred4_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16) >>> 0;
        const val2 = BigInt.asIntN(64, game);
        const val3 = +index;
        const ptr0 = passStringToWasm0(secret, wasm.__wbindgen_export3, wasm.__wbindgen_export4);
        const len0 = WASM_VECTOR_LEN;
        wasm.join_proof(retptr, val2, val3, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred4_0 = r0;
        deferred4_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export5(deferred4_0, deferred4_1, 1);
    }
}

/**
 * @returns {string}
 */
export function new_secret() {
    let deferred1_0;
    let deferred1_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16) >>> 0;
        wasm.new_secret(retptr);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred1_0 = r0;
        deferred1_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export5(deferred1_0, deferred1_1, 1);
    }
}

/**
 * Returns `votes|proof`. `pick` is ignored unless this seat holds the wolf. Use -1 for no pick.
 * @param {bigint} game
 * @param {number} index
 * @param {string} secret
 * @param {number} round
 * @param {string} joint
 * @param {string} residual
 * @param {Uint32Array | number[]} targets
 * @param {number} seats
 * @param {number} pick
 * @returns {string}
 */
export function night(game, index, secret, round, joint, residual, targets, seats, pick) {
    let deferred10_0;
    let deferred10_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16) >>> 0;
        const val5 = BigInt.asIntN(64, game);
        const val6 = +index;
        const val7 = +round;
        const val8 = +seats;
        const val9 = +pick;
        let ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3;
        try {
            ptr0 = passStringToWasm0(secret, wasm.__wbindgen_export3, wasm.__wbindgen_export4);
            len0 = WASM_VECTOR_LEN;
            ptr1 = passStringToWasm0(joint, wasm.__wbindgen_export3, wasm.__wbindgen_export4);
            len1 = WASM_VECTOR_LEN;
            ptr2 = passStringToWasm0(residual, wasm.__wbindgen_export3, wasm.__wbindgen_export4);
            len2 = WASM_VECTOR_LEN;
            ptr3 = passArray32ToWasm0(targets, wasm.__wbindgen_export3);
            len3 = WASM_VECTOR_LEN;
        } catch (e) {
            if (ptr3) { wasm.__wbindgen_export5(ptr3, len3 * 4, 4); }
            if (ptr2) { wasm.__wbindgen_export5(ptr2, len2 * 1, 1); }
            if (ptr1) { wasm.__wbindgen_export5(ptr1, len1 * 1, 1); }
            if (ptr0) { wasm.__wbindgen_export5(ptr0, len0 * 1, 1); }
            throw e;
        }
        wasm.night(retptr, val5, val6, ptr0, len0, val7, ptr1, len1, ptr2, len2, ptr3, len3, val8, val9);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred10_0 = r0;
        deferred10_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export5(deferred10_0, deferred10_1, 1);
    }
}

/**
 * @param {bigint} game
 * @param {number} index
 * @param {string} secret
 * @param {string} action
 * @param {string} sealer
 * @returns {string}
 */
export function payout_proof(game, index, secret, action, sealer) {
    let deferred6_0;
    let deferred6_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16) >>> 0;
        const val4 = BigInt.asIntN(64, game);
        const val5 = +index;
        let ptr0, len0, ptr1, len1, ptr2, len2;
        try {
            ptr0 = passStringToWasm0(secret, wasm.__wbindgen_export3, wasm.__wbindgen_export4);
            len0 = WASM_VECTOR_LEN;
            ptr1 = passStringToWasm0(action, wasm.__wbindgen_export3, wasm.__wbindgen_export4);
            len1 = WASM_VECTOR_LEN;
            ptr2 = passStringToWasm0(sealer, wasm.__wbindgen_export3, wasm.__wbindgen_export4);
            len2 = WASM_VECTOR_LEN;
        } catch (e) {
            if (ptr2) { wasm.__wbindgen_export5(ptr2, len2 * 1, 1); }
            if (ptr1) { wasm.__wbindgen_export5(ptr1, len1 * 1, 1); }
            if (ptr0) { wasm.__wbindgen_export5(ptr0, len0 * 1, 1); }
            throw e;
        }
        wasm.payout_proof(retptr, val4, val5, ptr0, len0, ptr1, len1, ptr2, len2);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred6_0 = r0;
        deferred6_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export5(deferred6_0, deferred6_1, 1);
    }
}

/**
 * @param {string} secret
 * @returns {string}
 */
export function public_key(secret) {
    let deferred2_0;
    let deferred2_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16) >>> 0;
        const ptr0 = passStringToWasm0(secret, wasm.__wbindgen_export3, wasm.__wbindgen_export4);
        const len0 = WASM_VECTOR_LEN;
        wasm.public_key(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred2_0 = r0;
        deferred2_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export5(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {bigint} game
 * @param {number} index
 * @param {string} secret
 * @param {string} residual
 * @returns {number}
 */
export function read_card(game, index, secret, residual) {
    const val3 = BigInt.asIntN(64, game);
    const val4 = +index;
    let ptr0, len0, ptr1, len1;
    try {
        ptr0 = passStringToWasm0(secret, wasm.__wbindgen_export3, wasm.__wbindgen_export4);
        len0 = WASM_VECTOR_LEN;
        ptr1 = passStringToWasm0(residual, wasm.__wbindgen_export3, wasm.__wbindgen_export4);
        len1 = WASM_VECTOR_LEN;
    } catch (e) {
        if (ptr1) { wasm.__wbindgen_export5(ptr1, len1 * 1, 1); }
        if (ptr0) { wasm.__wbindgen_export5(ptr0, len0 * 1, 1); }
        throw e;
    }
    const ret = wasm.read_card(val3, val4, ptr0, len0, ptr1, len1);
    return ret;
}

/**
 * Returns `shares|proof`.
 * @param {bigint} game
 * @param {number} index
 * @param {string} secret
 * @param {string} action
 * @param {number} round
 * @param {string} bases
 * @returns {string}
 */
export function shares(game, index, secret, action, round, bases) {
    let deferred7_0;
    let deferred7_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16) >>> 0;
        const val4 = BigInt.asIntN(64, game);
        const val5 = +index;
        const val6 = +round;
        let ptr0, len0, ptr1, len1, ptr2, len2;
        try {
            ptr0 = passStringToWasm0(secret, wasm.__wbindgen_export3, wasm.__wbindgen_export4);
            len0 = WASM_VECTOR_LEN;
            ptr1 = passStringToWasm0(action, wasm.__wbindgen_export3, wasm.__wbindgen_export4);
            len1 = WASM_VECTOR_LEN;
            ptr2 = passStringToWasm0(bases, wasm.__wbindgen_export3, wasm.__wbindgen_export4);
            len2 = WASM_VECTOR_LEN;
        } catch (e) {
            if (ptr2) { wasm.__wbindgen_export5(ptr2, len2 * 1, 1); }
            if (ptr1) { wasm.__wbindgen_export5(ptr1, len1 * 1, 1); }
            if (ptr0) { wasm.__wbindgen_export5(ptr0, len0 * 1, 1); }
            throw e;
        }
        wasm.shares(retptr, val4, val5, ptr0, len0, ptr1, len1, val6, ptr2, len2);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred7_0 = r0;
        deferred7_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export5(deferred7_0, deferred7_1, 1);
    }
}

/**
 * Every switch output and proof, in network order.
 * @param {bigint} game
 * @param {number} index
 * @param {string} secret
 * @param {string} deck
 * @param {string} joint
 * @returns {string}
 */
export function shuffle(game, index, secret, deck, joint) {
    let deferred6_0;
    let deferred6_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16) >>> 0;
        const val4 = BigInt.asIntN(64, game);
        const val5 = +index;
        let ptr0, len0, ptr1, len1, ptr2, len2;
        try {
            ptr0 = passStringToWasm0(secret, wasm.__wbindgen_export3, wasm.__wbindgen_export4);
            len0 = WASM_VECTOR_LEN;
            ptr1 = passStringToWasm0(deck, wasm.__wbindgen_export3, wasm.__wbindgen_export4);
            len1 = WASM_VECTOR_LEN;
            ptr2 = passStringToWasm0(joint, wasm.__wbindgen_export3, wasm.__wbindgen_export4);
            len2 = WASM_VECTOR_LEN;
        } catch (e) {
            if (ptr2) { wasm.__wbindgen_export5(ptr2, len2 * 1, 1); }
            if (ptr1) { wasm.__wbindgen_export5(ptr1, len1 * 1, 1); }
            if (ptr0) { wasm.__wbindgen_export5(ptr0, len0 * 1, 1); }
            throw e;
        }
        wasm.shuffle(retptr, val4, val5, ptr0, len0, ptr1, len1, ptr2, len2);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred6_0 = r0;
        deferred6_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export5(deferred6_0, deferred6_1, 1);
    }
}

/**
 * @param {bigint} game
 * @param {number} index
 * @param {string} secret
 * @param {string} steps
 * @returns {string}
 */
export function shuffle_proof(game, index, secret, steps) {
    let deferred5_0;
    let deferred5_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16) >>> 0;
        const val3 = BigInt.asIntN(64, game);
        const val4 = +index;
        let ptr0, len0, ptr1, len1;
        try {
            ptr0 = passStringToWasm0(secret, wasm.__wbindgen_export3, wasm.__wbindgen_export4);
            len0 = WASM_VECTOR_LEN;
            ptr1 = passStringToWasm0(steps, wasm.__wbindgen_export3, wasm.__wbindgen_export4);
            len1 = WASM_VECTOR_LEN;
        } catch (e) {
            if (ptr1) { wasm.__wbindgen_export5(ptr1, len1 * 1, 1); }
            if (ptr0) { wasm.__wbindgen_export5(ptr0, len0 * 1, 1); }
            throw e;
        }
        wasm.shuffle_proof(retptr, val3, val4, ptr0, len0, ptr1, len1);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred5_0 = r0;
        deferred5_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export5(deferred5_0, deferred5_1, 1);
    }
}

/**
 * @param {bigint} game
 * @param {string} secret
 * @param {number} nonce
 * @param {string} sealer
 * @returns {string}
 */
export function ticket_proof(game, secret, nonce, sealer) {
    let deferred5_0;
    let deferred5_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16) >>> 0;
        const val3 = BigInt.asIntN(64, game);
        const val4 = +nonce;
        let ptr0, len0, ptr1, len1;
        try {
            ptr0 = passStringToWasm0(secret, wasm.__wbindgen_export3, wasm.__wbindgen_export4);
            len0 = WASM_VECTOR_LEN;
            ptr1 = passStringToWasm0(sealer, wasm.__wbindgen_export3, wasm.__wbindgen_export4);
            len1 = WASM_VECTOR_LEN;
        } catch (e) {
            if (ptr1) { wasm.__wbindgen_export5(ptr1, len1 * 1, 1); }
            if (ptr0) { wasm.__wbindgen_export5(ptr0, len0 * 1, 1); }
            throw e;
        }
        wasm.ticket_proof(retptr, val3, ptr0, len0, val4, ptr1, len1);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred5_0 = r0;
        deferred5_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export5(deferred5_0, deferred5_1, 1);
    }
}

/**
 * Returns `votes|proof`.
 * @param {bigint} game
 * @param {number} index
 * @param {string} secret
 * @param {number} round
 * @param {string} joint
 * @param {Uint32Array | number[]} targets
 * @param {number} seats
 * @param {number} pick
 * @returns {string}
 */
export function vote(game, index, secret, round, joint, targets, seats, pick) {
    let deferred9_0;
    let deferred9_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16) >>> 0;
        const val4 = BigInt.asIntN(64, game);
        const val5 = +index;
        const val6 = +round;
        const val7 = +seats;
        const val8 = +pick;
        let ptr0, len0, ptr1, len1, ptr2, len2;
        try {
            ptr0 = passStringToWasm0(secret, wasm.__wbindgen_export3, wasm.__wbindgen_export4);
            len0 = WASM_VECTOR_LEN;
            ptr1 = passStringToWasm0(joint, wasm.__wbindgen_export3, wasm.__wbindgen_export4);
            len1 = WASM_VECTOR_LEN;
            ptr2 = passArray32ToWasm0(targets, wasm.__wbindgen_export3);
            len2 = WASM_VECTOR_LEN;
        } catch (e) {
            if (ptr2) { wasm.__wbindgen_export5(ptr2, len2 * 4, 4); }
            if (ptr1) { wasm.__wbindgen_export5(ptr1, len1 * 1, 1); }
            if (ptr0) { wasm.__wbindgen_export5(ptr0, len0 * 1, 1); }
            throw e;
        }
        wasm.vote(retptr, val4, val5, ptr0, len0, val6, ptr1, len1, ptr2, len2, val7, val8);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred9_0 = r0;
        deferred9_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export5(deferred9_0, deferred9_1, 1);
    }
}
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_is_function_741ff2bb612836f5: function(arg0) {
            const ret = typeof(arg0) === 'function';
            return ret;
        },
        __wbg___wbindgen_is_object_8b968c3ab8e96595: function(arg0) {
            const val = arg0;
            const ret = typeof(val) === 'object' && val !== null;
            return ret;
        },
        __wbg___wbindgen_is_string_fc456fee5d3317de: function(arg0) {
            const ret = typeof(arg0) === 'string';
            return ret;
        },
        __wbg___wbindgen_is_undefined_be5b34249ff1fc15: function(arg0) {
            const ret = arg0 === undefined;
            return ret;
        },
        __wbg___wbindgen_throw_4c88d8bf264fd984: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg_call_a410eb4b59ed5965: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.call(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_crypto_23f565a907d30078: function(arg0) {
            const ret = arg0.crypto;
            return ret;
        },
        __wbg_getRandomValues_d2908e0b99612b91: function() { return handleError(function (arg0, arg1) {
            arg0.getRandomValues(arg1);
        }, arguments); },
        __wbg_length_186163212ffe79c3: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_msCrypto_d5f06fad6bf1409b: function(arg0) {
            const ret = arg0.msCrypto;
            return ret;
        },
        __wbg_new_with_length_01b143a7e5dd5a3e: function(arg0) {
            const ret = new Uint8Array(arg0 >>> 0);
            return ret;
        },
        __wbg_node_86166ff8ce1f2002: function(arg0) {
            const ret = arg0.node;
            return ret;
        },
        __wbg_process_91f425a63f58e399: function(arg0) {
            const ret = arg0.process;
            return ret;
        },
        __wbg_prototypesetcall_13162965d1fe7aa3: function(arg0, arg1, arg2) {
            Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
        },
        __wbg_randomFillSync_34b023059e3363de: function() { return handleError(function (arg0, arg1) {
            arg0.randomFillSync(arg1);
        }, arguments); },
        __wbg_require_9a7d8cc2e1d275c0: function() { return handleError(function () {
            const ret = module.require;
            return ret;
        }, arguments); },
        __wbg_static_accessor_GLOBAL_THIS_003ee833af9699fc: function() {
            const ret = typeof globalThis === 'undefined' ? undefined : globalThis;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_GLOBAL_e4293fb22cbe77da: function() {
            const ret = typeof global === 'undefined' ? undefined : global;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_SELF_53bb20457401453d: function() {
            const ret = typeof self === 'undefined' ? undefined : self;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_WINDOW_b5e0810bfecb7feb: function() {
            const ret = typeof window === 'undefined' ? undefined : window;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_subarray_58f388a9247ac0bb: function(arg0, arg1, arg2) {
            const ret = arg0.subarray(arg1 >>> 0, arg2 >>> 0);
            return ret;
        },
        __wbg_versions_6e128a981b0df859: function(arg0) {
            const ret = arg0.versions;
            return ret;
        },
        __wbindgen_generic_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Ref(Slice(U8)) -> NamedExternref("Uint8Array")`.
            const ret = getArrayU8FromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_generic_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            table.set(0, undefined);
            table.set(1024, undefined);
            table.set(1025, null);
            table.set(1026, true);
            table.set(1027, false);
        },
    };
    return {
        __proto__: null,
        "./danse_client_bg.js": import0,
    };
}

function addToExternrefTable0(obj) {
    const idx = wasm.__wbindgen_export2();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

let cachedUint32ArrayMemory0 = null;
function getUint32ArrayMemory0() {
    if (cachedUint32ArrayMemory0 === null || cachedUint32ArrayMemory0.byteLength === 0) {
        cachedUint32ArrayMemory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32ArrayMemory0;
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_export(idx);
    }
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function passArray32ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 4, 4) >>> 0;
    getUint32ArrayMemory0().set(arg, ptr / 4);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(bytes) {
    numBytesDecoded += bytes.length;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true });
        cachedTextDecoder.decode();
        numBytesDecoded = bytes.length;
    }
    return cachedTextDecoder.decode(bytes);
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedUint32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (!module.ok) {
            throw new Error(`failed to fetch Wasm: ${module.status} ${module.statusText} fetching '${module.url}'`);
        }

        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

let __wbg_init_promise;
async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;
    if (__wbg_init_promise === undefined) {
        __wbg_init_promise = __wbg_init_once(module_or_path);
        __wbg_init_promise.catch(() => { __wbg_init_promise = undefined; });
    }
    return __wbg_init_promise;
}

async function __wbg_init_once(module_or_path) {

    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('danse_client_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
