/* tslint:disable */
/* eslint-disable */

export function ante_proof(game: bigint, index: number, secret: string, payout_key: string): string;

export function fee_proof(game: bigint, index: number, secret: string, nonce: number, sealer: string): string;

export function invite_proof(game: bigint, ticket: string, index: number, key: string): string;

export function join_proof(game: bigint, index: number, secret: string): string;

export function new_secret(): string;

/**
 * Returns `votes|proof`. `pick` is ignored unless this seat holds the wolf. Use -1 for no pick.
 */
export function night(game: bigint, index: number, secret: string, round: number, joint: string, residual: string, targets: Uint32Array | number[], seats: number, pick: number): string;

export function payout_proof(game: bigint, index: number, secret: string, action: string, sealer: string): string;

export function public_key(secret: string): string;

export function read_card(game: bigint, index: number, secret: string, residual: string): number;

/**
 * Returns `shares|proof`.
 */
export function shares(game: bigint, index: number, secret: string, action: string, round: number, bases: string): string;

/**
 * Every switch output and proof, in network order.
 */
export function shuffle(game: bigint, index: number, secret: string, deck: string, joint: string): string;

export function shuffle_proof(game: bigint, index: number, secret: string, steps: string): string;

export function ticket_proof(game: bigint, secret: string, nonce: number, sealer: string): string;

/**
 * Returns `votes|proof`.
 */
export function vote(game: bigint, index: number, secret: string, round: number, joint: string, targets: Uint32Array | number[], seats: number, pick: number): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly ante_proof: (a: number, b: bigint, c: number, d: number, e: number, f: number, g: number) => void;
    readonly fee_proof: (a: number, b: bigint, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly invite_proof: (a: number, b: bigint, c: number, d: number, e: number, f: number, g: number) => void;
    readonly join_proof: (a: number, b: bigint, c: number, d: number, e: number) => void;
    readonly new_secret: (a: number) => void;
    readonly night: (a: number, b: bigint, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number) => void;
    readonly payout_proof: (a: number, b: bigint, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly public_key: (a: number, b: number, c: number) => void;
    readonly read_card: (a: bigint, b: number, c: number, d: number, e: number, f: number) => number;
    readonly shares: (a: number, b: bigint, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => void;
    readonly shuffle: (a: number, b: bigint, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly shuffle_proof: (a: number, b: bigint, c: number, d: number, e: number, f: number, g: number) => void;
    readonly ticket_proof: (a: number, b: bigint, c: number, d: number, e: number, f: number, g: number) => void;
    readonly vote: (a: number, b: bigint, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number) => void;
    readonly __wbindgen_export: (a: number) => void;
    readonly __wbindgen_export2: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export3: (a: number, b: number) => number;
    readonly __wbindgen_export4: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export5: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
