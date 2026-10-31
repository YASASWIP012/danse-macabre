// Public network facts. Nothing here is secret.
export const INDEXER = "https://ootle-indexer-a.tari.com";
export const EXPLORER = "https://explorer.tari.mw";
export const COMPONENT = import.meta.env.VITE_COMPONENT as string;
export const TEMPLATE = import.meta.env.VITE_TEMPLATE as string;
export const SEATS = 5;
/** µT allowed per transaction. The engine refunds what isn't used. */
export const MAX_FEE = 3_000_000n;
