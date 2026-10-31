"""Turns a minotari burn proof file into the claim the Ootle engine reads."""
import base64, json, sys

src = json.load(open(sys.argv[1]))
claim = src["claim_proof"]
proof = claim["output_proof"]
out = proof["output"]
b64 = lambda h: base64.b64encode(bytes.fromhex(h)).decode()


def features(text):
    b = bytes.fromhex(text)
    p = 0

    def take(n):
        nonlocal p
        p += n
        return b[p - n:p]

    def vec():
        return take(int.from_bytes(take(4), "little"))

    version = take(1)[0]
    kind = take(1)[0]
    maturity = int.from_bytes(take(8), "little")
    vec()
    assert kind == 2, "not a burn"
    assert take(1)[0] == 1 and take(1)[0] == 2, "no confidential output feature"
    key = vec().hex()
    assert take(1)[0] == 0, "sidechain ids are not used here"
    range_proof = take(1)[0]
    assert p == len(b), "trailing bytes"
    return {"version": version, "maturity": str(maturity), "claim_public_key": key, "sidechain_id": None, "range_proof_type": range_proof}


def signature(sig):
    parts = [sig[k] for k in ("ephemeral_commitment", "ephemeral_pubkey", "u_a", "u_x", "u_y")]
    return base64.b64encode(b"".join(b"\x20\0\0\0" + bytes.fromhex(x) for x in parts)).decode()


def mmr(m):
    return {"leaf_index": str(m["leaf_index"]), "mmr_size": str(m["mmr_size"]), "path": m["path"], "peaks": m["peaks"]}


json.dump(
    {
        "claim_proof": {
            "commitment": out["commitment"],
            "ownership_proof": claim["ownership_proof"],
            "value": str(claim["value"]),
            "output": {
                "version": out["version"],
                "features": features(out["features"]),
                "rangeproof_hash": out["rangeproof_hash"],
                "script": b64(out["script"]),
                "sender_offset_public_key": out["sender_offset_public_key"],
                "metadata_signature": signature(out["metadata_signature"]),
                "covenant": b64(out["covenant"]),
                "encrypted_data": b64(out["encrypted_data"]),
                "minimum_value_promise": str(out["minimum_value_promise"]),
            },
            "inclusion_proof": {
                "block_hash": proof["block_hash"],
                "normal_output_proof": mmr(proof["normal_output_proof"]),
                "normal_output_mr": proof["normal_output_mr"],
                "block_output_proof": mmr(proof["block_output_proof"]),
            },
        },
        "encrypted_data": base64.b64decode(src["encrypted_data"]).hex(),
    },
    sys.stdout,
)
