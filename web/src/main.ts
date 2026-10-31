import "./style.css";
import init, * as w from "./wasm/danse_client";
import { hex, int, nextTableId, readTable, send, type TableState } from "./chain";
import { CARD_WOLF, PHASE, act, cardOf, isAlive, joinSeats, living, type Choices, type Seat } from "./game";
import { EXPLORER, SEATS } from "./net";

const $ = (id: string) => document.getElementById(id)!;
const NAMES = ["The Widow", "The Sexton", "The Miller", "The Bellringer", "The Gravedigger"];

let me = 0; // this player's seat
let ghosts: Seat[] = []; // seats this tab plays for
let mine: Seat | null = null;
let table: bigint | null = null;
let myCard = 0;

const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));
const say = (text: string) => ($("status").textContent = text);
const name = (i: number) => (i === me ? "You" : NAMES[i]);

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
      el.classList.toggle("you", i === me);
      if (t) {
        el.classList.toggle("dead", !isAlive(t, i));
        el.classList.toggle("acted", (t.acted & (1 << i)) !== 0);
      }
      const faceUp = i === me && myCard ? (myCard === CARD_WOLF ? "wolf" : "villager") : t && !isAlive(t, i) ? (t.wolf === i ? "wolf" : "villager") : "back";
      el.style.backgroundImage = `url(/art/card-${faceUp}.svg)`;
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
        b.textContent = seat === -1 ? "Start now" : name(seat);
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
const seat = (index: number): Seat => ({ index, secret: w.new_secret() });

/** Opens an empty table and returns its invite ticket. */
async function openTable(): Promise<string> {
  say("Opening a table on the testnet…");
  const ticket = w.new_secret();
  for (let attempt = 0; ; attempt++) {
    table = await nextTableId();
    try {
      const sent = await send({ kind: "open" }, [{ method: "open_table", args: [hex(w.public_key(ticket)), int(0)] }]);
      log("Table opened", sent.id);
      $("crumb").textContent = `Table ${table}`;
      return ticket;
    } catch (e) {
      if (attempt === 3) throw e;
      await wait(1500);
    }
  }
}

const LINES: Record<number, string> = {
  [PHASE.SHUFFLE]: "Seats shuffle, with a proof for every switch",
  [PHASE.DEAL]: "Seats strip their layers from the others' cards",
  [PHASE.NIGHT]: "Sealed night actions, all the same size",
  [PHASE.NIGHT_OPEN]: "The night's total opens",
  [PHASE.DAY]: "Sealed day votes",
  [PHASE.DAY_OPEN]: "The day's total opens",
  [PHASE.REVEAL]: "The dead seat turns its card face up",
};

/** Plays until the table ends. Every tab does the same: read, act for its own seats, repeat. */
async function play() {
  let lastPhase = -1;
  let announced = "";
  for (;;) {
    const t = await readTable(table!);
    if (!t) throw new Error("The table is gone");
    if (mine && !myCard && t.phase >= PHASE.NIGHT) {
      myCard = cardOf(t, mine);
      say(myCard === CARD_WOLF ? "You drew the wolf. Only you know that." : "You drew a villager. One of the others is the wolf.");
      drawSeats(t);
      await wait(2800);
    }
    drawSeats(t);
    if (t.phase === PHASE.OVER) {
      say(t.winner === 1 ? "The wolf is dead. The village wins." : t.winner === 2 ? "Only two remain. The wolf wins." : "The table stalled and closed.");
      if (t.wolf !== 255) log(`${name(t.wolf)} held the wolf.`);
      return;
    }
    const key = `${t.phase}/${t.round}`;
    if (t.phase === PHASE.REVEAL && announced !== key) {
      announced = key;
      say(`${name(t.turn)} ${t.turn === me ? "are" : "is"} dead.`);
      await wait(1500);
    }

    const local = [mine, ...ghosts].filter((s): s is Seat => s !== null);
    const others = living(t).filter((i) => i !== me);
    const choices: Choices = {};
    if (t.phase === PHASE.NIGHT) {
      const wolf = local.find((s) => cardOf(t, s) === CARD_WOLF);
      if (wolf && wolf === mine && isAlive(t, me) && !(t.acted & (1 << me))) {
        choices.kill = await ask("Choose who dies tonight.", others);
      } else if (wolf) {
        choices.kill = pickRandom(living(t).filter((i) => i !== wolf.index));
      }
      if (announced !== key) say("Night falls. Every living seat posts one sealed action. Only one of them means anything.");
      announced = key;
    }
    if (t.phase === PHASE.DAY) {
      const votes = new Map<number, number>();
      if (mine && isAlive(t, me) && !(t.acted & (1 << me))) votes.set(me, await ask("Who do you accuse?", others));
      for (const g of ghosts) if (isAlive(t, g.index)) votes.set(g.index, pickRandom(living(t).filter((i) => i !== g.index)));
      choices.votes = votes;
      if (announced !== key) say("The village votes in secret.");
      announced = key;
    }
    if (t.phase === PHASE.SHUFFLE && lastPhase !== t.phase) say("The seats shuffle one wolf into the deck. Nobody sees the order.");
    if (t.phase === PHASE.DEAL && lastPhase !== t.phase) say("Each seat strips its layer. Five cards, five owners.");
    if (t.phase === PHASE.NIGHT_OPEN && lastPhase !== t.phase) say("Dawn. Only the total opens.");
    lastPhase = t.phase;

    const live = local.filter((s) => t.phase === PHASE.REVEAL || t.phase === PHASE.SHUFFLE || isAlive(t, s.index));
    let sent = null;
    try {
      sent = live.length ? await act(t, live, choices) : null;
    } catch (e) {
      console.warn("retrying", e);
    }
    if (sent) log(LINES[t.phase] ?? "Move", sent.id);
    else await wait(2500);
  }
}

async function solo() {
  const ticket = await openTable();
  const seats = [0, 1, 2, 3, 4].map(seat);
  mine = seats[0];
  ghosts = seats.slice(1);
  const sent = await joinSeats(table!, ticket, seats);
  log("Five keys registered", sent.id);
  await play();
}

/** Opens a table, shares the link, and fills the empty seats with ghosts when the host says go. */
async function hostGroup() {
  const ticket = await openTable();
  mine = seat(0);
  await joinSeats(table!, ticket, [mine]);
  const link = `${location.origin}${location.pathname}#t=${table}&k=${ticket}`;
  history.replaceState(null, "", "#table");
  await navigator.clipboard?.writeText(link).catch(() => undefined);
  say("Table open. The invite link is copied. Send it to up to four friends.");
  log("You sit down. The link holds the ticket, so share it only with players.");
  for (;;) {
    const t = await readTable(table!);
    drawSeats(t ?? undefined);
    const free = [0, 1, 2, 3, 4].filter((i) => !(t!.joined & (1 << i)));
    if (!free.length) break;
    const go = await Promise.race([ask(`${5 - free.length} of 5 seated. Press to fill the rest with ghosts.`, [-1]), wait(4000).then(() => -2)]);
    if (go === -1) {
      ghosts = free.map(seat);
      await joinSeats(table!, ticket, ghosts);
      break;
    }
  }
  $("choices").replaceChildren();
  await play();
}

async function joinGroup(id: bigint, ticket: string) {
  table = id;
  $("crumb").textContent = `Table ${id}`;
  for (let attempt = 0; attempt < 6; attempt++) {
    const t = await readTable(id);
    if (!t || t.phase !== PHASE.LOBBY) throw new Error("This table has already started.");
    const free = [0, 1, 2, 3, 4].find((i) => !(t.joined & (1 << i)));
    if (free === undefined) throw new Error("This table is full.");
    me = free;
    mine = seat(free);
    try {
      await joinSeats(id, ticket, [mine]);
      say("You're seated. Waiting for the others.");
      await play();
      return;
    } catch {
      await wait(2000);
    }
  }
  throw new Error("Couldn't get a seat.");
}

async function guarded(run: () => Promise<void>) {
  for (const id of ["play", "friends"]) ($(id) as HTMLButtonElement).disabled = true;
  $("table").scrollIntoView({ behavior: "smooth", block: "center" });
  try {
    await run();
  } catch (e) {
    console.error(e);
    say("The testnet refused that. Refresh to try again.");
    log(String(e).slice(0, 200));
  }
}

async function start() {
  await init();
  drawSeats();
  $("play").addEventListener("click", () => guarded(solo));
  $("friends").addEventListener("click", () => guarded(hostGroup));
  const invite = new URLSearchParams(location.hash.slice(1));
  if (invite.get("t") && invite.get("k")) {
    const [id, ticket] = [BigInt(invite.get("t")!), invite.get("k")!];
    history.replaceState(null, "", "#table");
    await guarded(() => joinGroup(id, ticket));
  }
  fetch("/proof.json")
    .then((r) => (r.ok ? r.json() : []))
    .then((items: { text: string; href: string }[]) => {
      $("proofs").replaceChildren(
        ...items.map((p) => {
          const li = document.createElement("li");
          const a = document.createElement("a");
          a.href = p.href;
          a.textContent = p.text;
          li.append(a);
          return li;
        }),
      );
    })
    .catch(() => undefined);
}

start();
