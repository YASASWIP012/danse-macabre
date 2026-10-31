import "./style.css";
import init, * as w from "./wasm/danse_client";
import { findTable, hex, int, readTable, send, tablesOpened, type TableState } from "./chain";
import { CARD_WOLF, PHASE, act, cardOf, isAlive, joinSeats, living, unmask, type Choices, type Seat } from "./game";
import { ANTE, EXPLORER, SEATS } from "./net";
import { claim, connectWallet, stake } from "./stake";

const $ = (id: string) => document.getElementById(id)!;
const NAMES = ["The Widow", "The Sexton", "The Miller", "The Bell", "The Digger"];

let me = 0; // this player's seat
let ghosts: Seat[] = []; // seats this tab plays for
let mine: Seat | null = null;
let table: bigint | null = null;
let myCard = 0;
let walletAddress: string | null = null;
let payoutSecret: string | null = null;

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

let shownCard = 0;
let shownDead = 31;

function setTime(t?: TableState) {
  const night = t && (t.phase === PHASE.NIGHT || t.phase === PHASE.SHUFFLE || t.phase === PHASE.DEAL);
  const dawn = t && (t.phase === PHASE.NIGHT_OPEN || t.phase === PHASE.REVEAL);
  ($("room") as HTMLElement).dataset.time = night ? "night" : dawn ? "dawn" : "day";
}

function drawSeats(t?: TableState) {
  setTime(t);
  const flipMine = myCard !== 0 && shownCard === 0;
  if (flipMine) shownCard = myCard;
  const justDied = t ? shownDead & ~t.alive : 0;
  if (t) shownDead = t.alive;
  $("seats").replaceChildren(
    ...[...Array(SEATS).keys()].map((i) => {
      const el = document.createElement("div");
      el.className = "seat";
      el.classList.toggle("you", i === me);
      if ((flipMine && i === me) || justDied & (1 << i)) el.classList.add("flip");
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

/** What a tab must keep to come back to its table after a refresh. */
interface Saved {
  table: string;
  me: number;
  mine: Seat | null;
  ghosts: Seat[];
  wallet: string | null;
  payout: string | null;
}
const SAVE_KEY = "danse-macabre/table";

function save() {
  try {
    const saved: Saved = { table: String(table), me, mine, ghosts, wallet: walletAddress, payout: payoutSecret };
    localStorage.setItem(SAVE_KEY, JSON.stringify(saved));
  } catch {
    // A private window has no storage, and the game still plays without it.
  }
}

function restore(): Saved | null {
  try {
    const text = localStorage.getItem(SAVE_KEY);
    return text ? (JSON.parse(text) as Saved) : null;
  } catch {
    return null;
  }
}

/** Opens an empty table and returns its invite ticket. */
async function openTable(): Promise<string> {
  say("Opening a table on the testnet…");
  const ticket = w.new_secret();
  const key = w.public_key(ticket);
  const sent = await send({ kind: "open" }, [{ method: "open_table", args: [hex(key), int(0)] }]);
  log("Table opened", sent.id);
  for (let look = 0; look < 20 && table === null; look++) {
    table = await findTable(key);
    if (table === null) await wait(1000);
  }
  if (table === null) throw new Error("The new table didn't appear");
  $("crumb").textContent = `Table ${table}`;
  return ticket;
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

/** Pays a staked seat what it's owed, into a fresh stealth output. */
async function settle(first: TableState) {
  if (!payoutSecret || !walletAddress || !mine) return;
  let t = first;
  while (t.winner === 2 && t.wolf === 255) {
    // The wolf has to turn its card over before anyone can be paid. If it never does, the table
    // can be closed about an hour later, and then the stake comes back.
    say("Waiting for the wolf to turn its card over.");
    const fresh = await readTable(t.id).catch(() => null);
    if (fresh) t = fresh;
    await wait(4000);
  }
  const won = (me === t.wolf) === (t.winner === 2);
  const stakers = [0, 1, 2, 3, 4].filter((i) => t.staked & (1 << i));
  const winners = stakers.filter((i) => (i === t.wolf) === (t.winner === 2));
  const voided = t.winner === 3 || !winners.length;
  if (!voided && !won) return;
  const amount = voided ? ANTE : (ANTE * BigInt(stakers.length)) / BigInt(winners.length);
  for (let attempt = 0; attempt < 4; attempt++) {
    try {
      const sent = await claim(t, mine, payoutSecret, voided ? "refund" : "collect", amount, walletAddress);
      log(`${Number(amount) / 1e6} tTARI paid into a fresh stealth output`, sent.id);
      return;
    } catch (e) {
      if (attempt === 3) throw e;
      await wait(2500);
    }
  }
}

async function maybeStake() {
  if (!($("stakeit") as HTMLInputElement).checked || !mine) return;
  walletAddress = await connectWallet();
  if (!walletAddress) return log("Sapient isn't ready, so this game has no stake.");
  say("Approve the stake in Sapient.");
  const t = await readTable(table!);
  payoutSecret = await stake(t!, mine);
  save();
  log("You staked 1 tTARI from a stealth coin");
}

/** The end of the game: who it was, and a link anyone can check or share. */
function showResult(t: TableState) {
  const wolf = t.wolf === 255 ? "Nobody" : name(t.wolf);
  const youWere = myCard === CARD_WOLF ? "You were the wolf." : "You were a villager.";
  const verb = wolf === "You" ? "were" : "was";
  const moves = $("log").querySelectorAll("a").length;
  const line = `${wolf} ${verb} the wolf. ${youWere} ${moves} moves on the Tari testnet, and nobody could see who did it until the end.`;
  const box = $("result");
  box.hidden = false;
  box.replaceChildren();
  const title = document.createElement("strong");
  title.textContent = t.winner === 1 ? "The village survived" : t.winner === 2 ? "The wolf took the village" : "The table went quiet";
  const text = document.createElement("p");
  text.textContent = line;
  const row = document.createElement("div");
  const share = document.createElement("a");
  share.href = `https://bsky.app/intent/compose?text=${encodeURIComponent(`${line} ${location.origin}`)}`;
  share.target = "_blank";
  share.rel = "noreferrer";
  share.textContent = "Share on Bluesky";
  const copy = document.createElement("button");
  copy.textContent = "Copy result";
  copy.onclick = () => navigator.clipboard?.writeText(`${line} ${location.origin}`);
  row.append(share, copy);
  box.append(title, text, row);
}

const local = () => [mine, ...ghosts].filter((s): s is Seat => s !== null);

/** Plays until the table ends. Every tab does the same: read, act for its own seats, repeat. */
async function play() {
  let lastPhase = -1;
  let announced = "";
  let failures = 0;
  for (;;) {
    let t: TableState | null = null;
    try {
      t = await readTable(table!);
    } catch {
      await wait(2500);
      continue;
    }
    if (!t) throw new Error("The table is gone");
    if (mine && !myCard && t.phase >= PHASE.NIGHT) {
      myCard = cardOf(t, mine);
      say(myCard === CARD_WOLF ? "You drew the wolf. Only you know that." : "You drew a villager. One of the others is the wolf.");
      drawSeats(t);
      await wait(2800);
    }
    drawSeats(t);
    if (t.phase === PHASE.OVER) {
      const hiddenWolf = t.winner === 2 && t.wolf === 255 ? local().find((s) => cardOf(t, s) === CARD_WOLF) : undefined;
      if (hiddenWolf) {
        const sent = await unmask(t, hiddenWolf);
        log("The wolf turns its card over", sent.id);
        continue;
      }
      say(t.winner === 1 ? "The wolf is dead. The village wins." : t.winner === 2 ? "Only two remain. The wolf wins." : "The table stalled and closed.");
      if (t.wolf !== 255) log(`${name(t.wolf)} held the wolf.`);
      showResult(t);
      await settle(t);
      return;
    }
    const key = `${t.phase}/${t.round}`;
    if (t.phase === PHASE.REVEAL && announced !== key) {
      announced = key;
      say(`${name(t.turn)} ${t.turn === me ? "are" : "is"} dead.`);
      await wait(1500);
    }

    const seated = local();
    const others = living(t).filter((i) => i !== me);
    const choices: Choices = {};
    if (t.phase === PHASE.NIGHT) {
      const wolf = seated.find((s) => cardOf(t, s) === CARD_WOLF);
      // Everyone picks a name at night, and a villager's pick goes nowhere. If only the wolf paused
      // to think, the pause would give them away.
      if (mine && isAlive(t, me) && !(t.acted & (1 << me))) {
        const pick = await ask(myCard === CARD_WOLF ? "Choose who dies tonight." : "Choose who you'd watch tonight. It changes nothing, and nobody can see that.", others);
        if (wolf === mine) choices.kill = pick;
        await wait(1000 + Math.random() * 4000);
      }
      if (wolf && wolf !== mine) choices.kill = pickRandom(living(t).filter((i) => i !== wolf.index));
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

    const live = seated.filter((s) => t.phase === PHASE.REVEAL || t.phase === PHASE.SHUFFLE || isAlive(t, s.index));
    let sent = null;
    try {
      sent = live.length ? await act(t, live, choices) : null;
      failures = 0;
    } catch (e) {
      console.warn("retrying", e);
      if (++failures === 5) throw e;
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
  save();
  log("Five keys registered", sent.id);
  await maybeStake();
  await play();
}

/** Opens a table, shares the link, and fills the empty seats with ghosts when the host says go. */
async function hostGroup() {
  const ticket = await openTable();
  mine = seat(0);
  await joinSeats(table!, ticket, [mine]);
  save();
  await maybeStake();
  const link = `${location.origin}${location.pathname}#t=${table}&k=${ticket}`;
  history.replaceState(null, "", "#table");
  await navigator.clipboard?.writeText(link).catch(() => undefined);
  say("Table open. The invite link is copied. Send it to up to four friends.");
  log("The link holds the ticket, so share it only with players.");
  log(link);
  for (;;) {
    const t = await readTable(table!);
    drawSeats(t ?? undefined);
    const free = [0, 1, 2, 3, 4].filter((i) => !(t!.joined & (1 << i)));
    if (!free.length) break;
    const go = await Promise.race([ask(`${5 - free.length} of 5 seated. Ghosts run in this tab, so you could read their cards.`, [-1]), wait(4000).then(() => -2)]);
    if (go === -1) {
      try {
        const fill = free.map(seat);
        await joinSeats(table!, ticket, fill);
        ghosts = fill;
        save();
        break;
      } catch {
        log("Someone sat down at the same moment. Try again.");
      }
    }
  }
  $("choices").replaceChildren();
  await play();
}

async function joinGroup(id: bigint, ticket: string) {
  table = id;
  $("crumb").textContent = `Table ${id}`;
  let seated = false;
  for (let attempt = 0; attempt < 6 && !seated; attempt++) {
    const t = await readTable(id);
    if (!t || t.phase !== PHASE.LOBBY) throw new Error("This table has already started.");
    const free = [0, 1, 2, 3, 4].find((i) => !(t.joined & (1 << i)));
    if (free === undefined) throw new Error("This table is full.");
    const candidate = seat(free);
    try {
      await joinSeats(id, ticket, [candidate]);
      me = free;
      mine = candidate;
      seated = true;
      save();
    } catch {
      await wait(2000);
    }
  }
  if (!seated) throw new Error("Couldn't get a seat.");
  await maybeStake();
  say("You're seated. Waiting for the others.");
  await play();
}

/** Picks a table back up after a refresh, using what this browser kept. */
async function resume(saved: Saved) {
  table = BigInt(saved.table);
  me = saved.me;
  mine = saved.mine;
  ghosts = saved.ghosts;
  walletAddress = saved.wallet;
  payoutSecret = saved.payout;
  $("crumb").textContent = `Table ${table}`;
  await play();
}

function clearTable() {
  table = null;
  me = 0;
  mine = null;
  ghosts = [];
  myCard = 0;
  shownCard = 0;
  shownDead = 31;
  payoutSecret = null;
  $("result").hidden = true;
  $("log").replaceChildren();
  $("choices").replaceChildren();
}

async function guarded(run: () => Promise<void>) {
  for (const id of ["play", "friends"]) ($(id) as HTMLButtonElement).disabled = true;
  $("table").scrollIntoView({ behavior: "smooth", block: "center" });
  try {
    await run();
  } catch (e) {
    console.error(e);
    say("The testnet refused that. Refresh, and your seat comes back.");
    log(String(e).slice(0, 200));
  }
  for (const id of ["play", "friends"]) ($(id) as HTMLButtonElement).disabled = false;
}

/** A refresh shouldn't drop a player back into an old table unasked, so this asks. */
function offerResume(saved: Saved) {
  say(`You have a seat at table ${saved.table}.`);
  const back = document.createElement("button");
  back.textContent = "Go back to it";
  back.onclick = () => {
    $("choices").replaceChildren();
    void guarded(() => resume(saved));
  };
  $("choices").replaceChildren(back);
}

async function start() {
  await init();
  drawSeats();
  tablesOpened()
    .then((n) => ($("count").textContent = `${n} tables opened so far.`))
    .catch(() => undefined);
  $("stakebox").hidden = !new URLSearchParams(location.search).has("stake");
  $("play").addEventListener("click", () => {
    clearTable();
    return guarded(solo);
  });
  $("friends").addEventListener("click", () => {
    clearTable();
    return guarded(hostGroup);
  });
  const invite = new URLSearchParams(location.hash.slice(1));
  if (invite.get("t") && invite.get("k")) {
    const [id, ticket] = [BigInt(invite.get("t")!), invite.get("k")!];
    history.replaceState(null, "", "#table");
    await guarded(() => joinGroup(id, ticket));
  }
  const saved = restore();
  if (saved && !invite.get("t")) {
    const old = await readTable(BigInt(saved.table)).catch(() => null);
    const unpaid = old && saved.payout && !(old.collected & (1 << saved.me));
    if (old && (old.phase !== PHASE.OVER || unpaid)) offerResume(saved);
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
