// A few sounds made in the browser, so the page loads nothing extra. They start after the first
// press, which is when browsers allow audio.
let context: AudioContext | null = null;
let muted = false;

export function toggleSound(): boolean {
  muted = !muted;
  return !muted;
}

function ctx(): AudioContext | null {
  if (muted) return null;
  context ??= new AudioContext();
  void context.resume();
  return context;
}

function tone(frequency: number, seconds: number, gain: number, delay = 0, type: OscillatorType = "sine") {
  const a = ctx();
  if (!a) return;
  const start = a.currentTime + delay;
  const osc = a.createOscillator();
  const amp = a.createGain();
  osc.type = type;
  osc.frequency.value = frequency;
  amp.gain.setValueAtTime(0, start);
  amp.gain.linearRampToValueAtTime(gain, start + 0.02);
  amp.gain.exponentialRampToValueAtTime(0.0001, start + seconds);
  osc.connect(amp).connect(a.destination);
  osc.start(start);
  osc.stop(start + seconds);
}

/** A church bell: a low strike with inharmonic partials. */
export function bell(strikes = 1) {
  for (let i = 0; i < strikes; i++) {
    const at = i * 1.5;
    for (const [ratio, level] of [[1, 0.16], [2.01, 0.08], [2.76, 0.05], [5.4, 0.02]]) tone(196 * ratio, 2.6, level, at);
  }
}

/** The card turning over. */
export function flipSound() {
  tone(740, 0.09, 0.05, 0, "triangle");
  tone(1100, 0.07, 0.03, 0.05, "triangle");
}

/** A low howl for the wolf's night. */
export function howl() {
  const a = ctx();
  if (!a) return;
  const osc = a.createOscillator();
  const amp = a.createGain();
  const t = a.currentTime;
  osc.frequency.setValueAtTime(220, t);
  osc.frequency.exponentialRampToValueAtTime(420, t + 0.9);
  osc.frequency.exponentialRampToValueAtTime(240, t + 2.4);
  amp.gain.setValueAtTime(0, t);
  amp.gain.linearRampToValueAtTime(0.07, t + 0.5);
  amp.gain.exponentialRampToValueAtTime(0.0001, t + 2.5);
  osc.connect(amp).connect(a.destination);
  osc.start(t);
  osc.stop(t + 2.5);
}
