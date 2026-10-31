// One player's view of a table: works out what the seats held in this tab owe, proves it and sends
// it as one transaction. In solo play this tab holds all five seats, and four of them are ghosts.
import { hex, int, send, type Call, type TableState } from "./chain";
import * as w from "./wasm/danse_client";
import { SEATS } from "./net";

export const PHASE = { LOBBY: 0, SHUFFLE: 1, DEAL: 2, NIGHT: 3, NIGHT_OPEN: 4, REVEAL: 5, DAY: 6, DAY_OPEN: 7, OVER: 8 } as const;
export const CARD_WOLF = 7;

export interface Seat {
  index: number;
  secret: string;
}

const cut = (s: string, size: number, index: number) => s.slice(index * size * 2, (index + 1) * size * 2);
export const isAlive = (t: TableState, seat: number) => (t.alive & (1 << seat)) !== 0;
const hasActed = (t: TableState, seat: number) => (t.acted & (1 << seat)) !== 0;
export const living = (t: TableState) => [...Array(SEATS).keys()].filter((i) => isAlive(t, i));

/** The card this seat was dealt, once dealing is finished. */
export function cardOf(t: TableState, seat: Seat): number {
  return w.read_card(t.id, seat.index, seat.secret, cut(t.residual, 64, seat.index));
}

export interface Choices {
  /** The wolf's victim tonight. */
  kill?: number;
  /** Each seat's day vote. */
  votes?: Map<number, number>;
}

function calls(t: TableState, seats: Seat[], choices: Choices): Call[] {
  const out: Call[] = [];
  const table = t.id;
  switch (t.phase) {
    case PHASE.SHUFFLE: {
      const seat = seats.find((s) => s.index === t.turn);
      if (!seat) break;
      const steps = w.shuffle(table, seat.index, seat.secret, t.deck, t.joint);
      out.push({ method: "shuffle", args: [int(table), int(seat.index), hex(steps)] });
      break;
    }
    case PHASE.DEAL:
      for (const seat of seats.filter((s) => !hasActed(t, s.index))) {
        const bases = [...Array(SEATS).keys()].filter((i) => i !== seat.index).map((i) => cut(t.deck, 64, i).slice(0, 64)).join("");
        const [shares, proof] = w.shares(table, seat.index, seat.secret, "deal", 0, bases).split("|");
        out.push({ method: "deal", args: [int(table), int(seat.index), hex(shares), hex(proof)] });
      }
      break;
    case PHASE.NIGHT:
      for (const seat of seats.filter((s) => isAlive(t, s.index) && !hasActed(t, s.index))) {
        const targets = Uint32Array.from(living(t).filter((i) => i !== seat.index));
        const pick = choices.kill !== undefined && choices.kill !== seat.index ? choices.kill : -1;
        const [votes, proof] = w.night(table, seat.index, seat.secret, t.round, t.joint, cut(t.residual, 64, seat.index), targets, SEATS, pick).split("|");
        out.push({ method: "night", args: [int(table), int(seat.index), hex(votes), hex(proof)] });
      }
      break;
    case PHASE.DAY:
      for (const seat of seats.filter((s) => isAlive(t, s.index) && !hasActed(t, s.index))) {
        const options = living(t).filter((i) => i !== seat.index);
        const pick = choices.votes?.get(seat.index) ?? options[0];
        const [votes, proof] = w.vote(table, seat.index, seat.secret, t.round, t.joint, Uint32Array.from(options), SEATS, pick).split("|");
        out.push({ method: "vote", args: [int(table), int(seat.index), hex(votes), hex(proof)] });
      }
      break;
    case PHASE.NIGHT_OPEN:
    case PHASE.DAY_OPEN: {
      const round = t.round + (t.phase === PHASE.DAY_OPEN ? 1 << 16 : 0);
      const bases = [...Array(SEATS).keys()].map((i) => cut(t.tally, 64, i).slice(0, 64)).join("");
      for (const seat of seats.filter((s) => isAlive(t, s.index) && !hasActed(t, s.index))) {
        const [shares, proof] = w.shares(table, seat.index, seat.secret, "open", round, bases).split("|");
        out.push({ method: "open_tally", args: [int(table), int(seat.index), hex(shares), hex(proof)] });
      }
      break;
    }
    case PHASE.REVEAL: {
      const seat = seats.find((s) => s.index === t.turn);
      if (seat) out.push({ method: "reveal", args: [int(table), int(seat.index), hex(seat.secret)] });
      break;
    }
  }
  return out;
}

/** Sends whatever these seats owe in this phase. Returns the transaction, or null if nothing is owed. */
export async function act(t: TableState, seats: Seat[], choices: Choices = {}) {
  const list = calls(t, seats, choices);
  if (!list.length) return null;
  const payer = seats[0];
  const proof = w.fee_proof(t.id, payer.index, payer.secret, t.feeNonce);
  return send({ kind: "seat", table: t.id, seat: payer.index, proof }, list);
}
