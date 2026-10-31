// Talks to Esmeralda. Every transaction is sealed by a throwaway key and paid for by the fee pot.
import { Network, TransactionBuilder, bytesLiteral, fromHexStr, intLiteral, resolveMaxEpoch, sendTransaction, toHexStr, type InstructionArg } from "@tari-project/ootle";
import { IndexerProvider } from "@tari-project/ootle-indexer";
import { EphemeralKeySigner } from "@tari-project/ootle-secret-key-wallet";
import { COMPONENT, INDEXER, MAX_FEE } from "./net";

export type Fee =
  | { kind: "open" }
  | { kind: "ticket"; table: bigint; proof: string }
  | { kind: "seat"; table: bigint; seat: number; proof: string };

export interface Call {
  method: string;
  args: InstructionArg[];
}

export const hex = (s: string) => bytesLiteral(fromHexStr(s));
export const int = (n: number | bigint) => intLiteral(BigInt(n));
export const toHex = toHexStr;

let provider: Promise<IndexerProvider> | null = null;
const connect = () => (provider ??= IndexerProvider.connect({ url: INDEXER, network: Network.Esmeralda }));

function feeCall(fee: Fee): Call {
  switch (fee.kind) {
    case "open":
      return { method: "pay_open_fee", args: [int(MAX_FEE)] };
    case "ticket":
      return { method: "pay_ticket_fee", args: [int(fee.table), hex(fee.proof), int(MAX_FEE)] };
    case "seat":
      return { method: "pay_seat_fee", args: [int(fee.table), int(fee.seat), hex(fee.proof), int(MAX_FEE)] };
  }
}

export interface Sent {
  id: string;
  ms: number;
}

/** Sends one transaction of `calls` and waits until it finalizes. */
export async function send(fee: Fee, calls: Call[]): Promise<Sent> {
  const started = performance.now();
  const p = await connect();
  const pay = feeCall(fee);
  let b = new TransactionBuilder(Network.Esmeralda, await resolveMaxEpoch(p)).withFeeInstructionsBuilder((f) =>
    f.callMethod({ componentAddress: COMPONENT as never, methodName: pay.method }, pay.args),
  );
  for (const call of calls) b = b.callMethod({ componentAddress: COMPONENT as never, methodName: call.method }, call.args);
  const signer = EphemeralKeySigner.generate(Network.Esmeralda);
  const result: any = await sendTransaction(p, signer, b.buildUnsignedTransaction());
  return { id: String(result?.transaction?.id ?? result?.transaction_id ?? ""), ms: performance.now() - started };
}

export interface TableState {
  id: bigint;
  phase: number;
  round: number;
  acted: number;
  alive: number;
  turn: number;
  afterReveal: number;
  winner: number;
  wolf: number;
  lastVictim: number;
  joined: number;
  keys: string;
  joint: string;
  deck: string;
  residual: string;
  tally: string;
  feeNonce: number;
}

const hexOf = (v: any): string => {
  if (typeof v === "string") return v.replace(/^0x/, "");
  if (v instanceof Uint8Array) return toHexStr(v);
  if (Array.isArray(v)) return toHexStr(Uint8Array.from(v));
  if (v && typeof v === "object" && "@bytes" in v) return String(v["@bytes"]);
  return "";
};

/** Reads a table straight from the component's public state. */
export async function readTable(id: bigint): Promise<TableState | null> {
  const r = await fetch(`${INDEXER}/substates/${COMPONENT}`, { cache: "no-store" });
  if (!r.ok) throw new Error(`The indexer answered ${r.status}`);
  const j = await r.json();
  const sub = j?.substate ?? j?.state?.up?.substate;
  const state = sub?.Component?.body?.state;
  const tables = Array.isArray(state) ? state[0] : state?.tables;
  const entry = (Array.isArray(tables) ? tables : Object.entries(tables ?? {})).find((e: any) => BigInt(e[0]) === id);
  if (!entry) return null;
  const t = entry[1];
  const f = (name: string, index: number) => (Array.isArray(t) ? t[index] : t[name]);
  return {
    id,
    phase: Number(f("phase", 1)),
    round: Number(f("round", 2)),
    acted: Number(f("acted", 3)),
    alive: Number(f("alive", 4)),
    turn: Number(f("turn", 5)),
    afterReveal: Number(f("after_reveal", 6)),
    winner: Number(f("winner", 7)),
    wolf: Number(f("wolf", 8)),
    lastVictim: Number(f("last_victim", 9)),
    keys: hexOf(f("keys", 10)),
    joined: Number(f("joined", 11)),
    joint: hexOf(f("joint", 12)),
    deck: hexOf(f("deck", 13)),
    residual: hexOf(f("residual", 14)),
    tally: hexOf(f("tally", 15)),
    feeNonce: Number(f("fee_nonce", 19)),
  };
}

/** Sends with retries, for the moments two players touch the table at once. */
export async function sendRetrying(build: () => Promise<{ fee: Fee; calls: Call[] }>, tries = 4): Promise<Sent> {
  let failure: unknown;
  for (let attempt = 0; attempt < tries; attempt++) {
    try {
      const { fee, calls } = await build();
      return await send(fee, calls);
    } catch (e) {
      failure = e;
      await new Promise((r) => setTimeout(r, 1500 + attempt * 1500));
    }
  }
  throw failure;
}

export async function nextTableId(): Promise<bigint> {
  const r = await fetch(`${INDEXER}/substates/${COMPONENT}`, { cache: "no-store" });
  const j = await r.json();
  const state = (j?.substate ?? j?.state?.up?.substate)?.Component?.body?.state;
  return BigInt(Array.isArray(state) ? state[1] : state?.next_table);
}
