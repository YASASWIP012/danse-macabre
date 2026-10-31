// The optional stake. It comes out of a Sapient stealth balance, so no wallet is tied to the seat,
// and winnings leave in one fresh stealth output, so the payout names nobody either.
import { Mask, Network, StealthTransferStatement, TARI_RESOURCE_ADDRESS, WasmStealthCrypto, createOutput, stealthTransferInstruction } from "@tari-project/ootle";
import { hex, int, readTable, send, type TableState } from "./chain";
import { COMPONENT, ANTE } from "./net";
import * as w from "./wasm/danse_client";
import type { Seat } from "./game";

declare global {
  interface Window {
    tari?: { request(args: { method: string; params?: unknown }): Promise<any> };
  }
}

const hasWallet = () => typeof window.tari?.request === "function";

/** The wallet's receiving address, or null if Sapient isn't there or can't spend privately. */
export async function connectWallet(): Promise<string | null> {
  if (!hasWallet()) return null;
  await window.tari!.request({ method: "tari_requestAccounts" });
  const caps = await window.tari!.request({ method: "tari_getCapabilities" });
  if (!caps?.stealthWithdraw) return null;
  return window.tari!.request({ method: "tari_getWalletAddress" });
}

/** Stakes one seat from the wallet's private balance. Returns the secret that claims the money. */
export async function stake(t: TableState, seat: Seat): Promise<string> {
  const payout = w.new_secret();
  const payoutKey = w.public_key(payout);
  const proof = w.ante_proof(t.id, seat.index, seat.secret, payoutKey);
  const tari = window.tari!;
  const { requestId } = await tari.request({
    method: "tari_createTransactionRequest",
    params: {
      kind: "withdrawStealthAndExecute",
      resourceAddress: TARI_RESOURCE_ADDRESS,
      amount: ANTE.toString(),
      workspaceVarName: "stake",
      followUpInstructions: [
        {
          CallMethod: {
            call: { Address: COMPONENT },
            method: "ante",
            args: [int(t.id), int(seat.index), hex(payoutKey), hex(proof), { Workspace: { id: 1, offset: null } }],
          },
        },
      ],
      relatedComponents: [COMPONENT],
      maxFee: "1000000",
    },
  });
  for (;;) {
    const state = await tari.request({ method: "tari_getTransactionRequest", params: { requestId } });
    if (state.status === "pending") {
      await new Promise((r) => setTimeout(r, 1500));
      continue;
    }
    if (state.status !== "approved") throw new Error(state.error ?? "The wallet declined the stake");
    break;
  }
  await tari.request({ method: "tari_submitTransactionRequest", params: { requestId } });
  for (let look = 0; look < 40; look++) {
    const now = await readTable(t.id);
    if (now && now.staked & (1 << seat.index)) return payout;
    await new Promise((r) => setTimeout(r, 1500));
  }
  throw new Error("The stake never landed");
}

/** Sends `amount` of the table's stakes to `address` as a new stealth output. */
export async function claim(t: TableState, seat: Seat, payout: string, action: "collect" | "refund", amount: bigint, address: string) {
  const crypto = new WasmStealthCrypto(Network.Esmeralda);
  const output = createOutput({ destination: address, amount, resourceAddress: TARI_RESOURCE_ADDRESS });
  const { statement: outputs, outputMask } = await crypto.generateOutputsStatement([output], null);
  const inputs = await crypto.buildInputsStatement([], amount);
  const balance = await crypto.generateBalanceProofSignature(Mask.zero(), outputMask, inputs.statementJson!, outputs.statementJson);
  const statement = new StealthTransferStatement(inputs, outputs, balance);
  const nonce = (await readTable(t.id))?.feeNonce ?? t.feeNonce;
  const fee = (sealer: string) => w.payout_fee_proof(t.id, seat.index, payout, nonce, sealer);
  return send(
    { kind: "payout", table: t.id, seat: seat.index, proof: fee },
    (sealer) => [{ method: action, args: [int(t.id), int(seat.index), hex(w.payout_proof(t.id, seat.index, payout, action, sealer))] }],
    (b) => {
      b.saveVar("won");
      return b
        .addInstruction(stealthTransferInstruction({ resourceAddress: TARI_RESOURCE_ADDRESS, revealedInputBucket: "won", statement }, (name) => b.resolveWorkspaceOffsetId(name)));
    },
  );
}
