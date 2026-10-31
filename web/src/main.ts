import "./style.css";
import init, * as w from "./wasm/danse_client";
import { hex, int, nextTableId, readTable, send, type Call, type TableState } from "./chain";
import { CARD_WOLF, PHASE, act, cardOf, isAlive, living, type Choices, type Seat } from "./game";
import { EXPLORER, SEATS } from "./net";

const $ = (id: string) => document.getElementById(id)!;
const NAMES = ["You", "The Widow", "The Sexton", "The Miller", "The Bellringer"];
const YOU = 0;

let seats: Seat[] = [];
let table: bigint | null = null;
let myCard = 0;

const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));
const say = (text: string) => ($("status").textContent = text);
const name = (i: number) => NAMES[i];

function log(text: string, tx?: string) {
  const li = document.createElement("li");
  li.textContent = text + " ";
  if (tx) {
    const a = document.createElement("a");
    a.href = `${EXPLORER}/tx/${tx}`;
    a.target = "_blank";
    a.rel = "noreferrer";
    a.textContent = tx.slice(0, 10) + "…";
    li.append(a);
  }
  $("log").prepend(li);
}

function drawSeats(t?: TableState) {
  $("seats").replaceChildren(
    ...[...Array(SEATS).keys()].map((i) => {
      const el = document.createElement("div");
      el.className = "seat";
      el.classList.toggle("you", i === YOU);
      if (t) {
        el.classList.toggle("dead", !isAlive(t, i));
        el.classList.toggle("acted", (t.acted & (1 << i)) !== 0);
      }
      const faceUp = i === YOU && myCard ? (myCard === CARD_WOLF ? "wolf" : "villager") : t && !isAlive(t, i) ? (t.wolf === i ? "wolf" : "villager") : "back";
      el.style.backgroundImage = `url(/art/card-${faceUp}.png)`;
      el.innerHTML = `<small>Seat ${i + 1}</small><span class="name">${name(i)}</span>`;
      return el;
    }),
  );
}

function ask(prompt: string, options: number[]): Promise<number> {
  say(prompt);
  return new Promise((resolve) => {
    $("choices").replaceChildren(
      ...options.map((seat) => {
        const b = document.createElement("button");
        b.textContent = name(seat);
        b.onclick = () => {
          $("choices").replaceChildren();
          resolve(seat);
        };
        return b;
      }),
    );
  });
}

const pickRandom = <T,>(list: T[]) => list[Math.floor(Math.random() * list.length)];

async function openTable() {
  say("Opening a table on the testnet…");
  const ticket = w.new_secret();
  seats = [...Array(SEATS).keys()].map((index) => ({ index, secret: w.new_secret() }));
  for (let attempt = 0; attempt < 4; attempt++) {
    table = await nextTableId();
    const calls: Call[] = [{ method: "open_table", args: [hex(w.public_key(ticket)), int(0)] }];
    for (const s of seats) {
      calls.push({ method: "join", args: [int(table), int(s.index), hex(w.public_key(s.secret)), hex(w.join_proof(table, s.index, s.secret))] });
    }
    try {
      const sent = await send({ kind: "open" }, calls);
      log("Table opened, five keys registered", sent.id);
      $("crumb").textContent = `Table ${table}`;
      return;
    } catch (e) {
      if (attempt === 3) throw e;
      await wait(1500);
    }
  }
}

const LINES: Record<number, string> = {
  [PHASE.SHUFFLE]: "Seat shuffles the deck, with a proof for every switch",
  [PHASE.DEAL]: "Every seat strips its layer from the others' cards",
  [PHASE.NIGHT]: "Five sealed actions of the same size",
  [PHASE.NIGHT_OPEN]: "The night's total opens",
  [PHASE.DAY]: "Sealed day votes",
  [PHASE.DAY_OPEN]: "The day's total opens",
  [PHASE.REVEAL]: "The dead seat turns its card face up",
};

async function run() {
  await openTable();
  let last = -1;
  for (;;) {
    const t = await readTable(table!);
    if (!t) throw new Error("The table is gone");
    if (!myCard && t.phase >= PHASE.NIGHT) {
      myCard = cardOf(t, seats[YOU]);
      say(myCard === CARD_WOLF ? "You drew the wolf. Only you know that." : "You drew a villager. One of these four is the wolf.");
      await wait(2800);
    }
    drawSeats(t);
    if (t.phase === PHASE.OVER) {
      say(t.winner === 1 ? "The wolf is dead. The village wins." : "Only two remain. The wolf wins.");
      log(`Game over. ${name(t.wolf)} ${t.wolf === 255 ? "" : "was the wolf."}`);
      return;
    }
    if (t.phase === PHASE.REVEAL && t.phase !== last) say(`${name(t.turn)} is dead.`);

    const others = living(t).filter((i) => i !== YOU);
    const choices: Choices = {};
    if (t.phase === PHASE.NIGHT) {
      if (myCard === CARD_WOLF && isAlive(t, YOU)) {
        choices.kill = await ask("Choose who dies tonight.", others);
      } else {
        const wolf = seats.find((s) => cardOf(t, s) === CARD_WOLF)!;
        choices.kill = pickRandom(living(t).filter((i) => i !== wolf.index));
        say("Night falls. Five sealed actions go on chain. Only one of them means anything.");
      }
    }
    if (t.phase === PHASE.DAY) {
      const votes = new Map<number, number>();
      if (isAlive(t, YOU)) votes.set(YOU, await ask("Who do you accuse?", others));
      for (const s of living(t)) if (s !== YOU) votes.set(s, pickRandom(living(t).filter((i) => i !== s)));
      choices.votes = votes;
      say("The village votes in secret.");
    }
    if (t.phase === PHASE.SHUFFLE) say(`${name(t.turn)} shuffles.`);
    if (t.phase === PHASE.DEAL) say("Each seat strips its layer. Five cards, five owners.");

    await wait(30);
    const sent = await act(t, seats, choices);
    if (sent) log(LINES[t.phase] ?? "Move", sent.id);
    last = t.phase;
  }
}

async function start() {
  await init();
  drawSeats();
  $("play").addEventListener("click", async () => {
    ($("play") as HTMLButtonElement).disabled = true;
    $("table").scrollIntoView({ behavior: "smooth", block: "center" });
    try {
      await run();
    } catch (e) {
      console.error(e);
      say("The testnet refused that move. Refresh to open a new table.");
      log(String(e).slice(0, 200));
    }
    ($("play") as HTMLButtonElement).disabled = false;
  });
}

start();
