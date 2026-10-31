// Public network facts. Nothing here is secret.
export const INDEXER = "https://ootle-indexer-a.tari.com";
export const EXPLORER = "https://explorer.tari.mw";
export const COMPONENT = import.meta.env.VITE_COMPONENT as string;
export const POT_VAULT = "vault_3c3c028ab88e9a1b9171a677a8be81686566ac69418462c8c279c8c6badd38d0";
export const STAKES_VAULT = "vault_3cac31b7c83e456eb3849f0a12d6b8013b06725e7c23a4c2df31de362dc3269d";
export const TEMPLATE = import.meta.env.VITE_TEMPLATE as string;
export const SEATS = 5;
/** The stake, in µT. It has to match the template. */
export const ANTE = 1_000_000n;
/** µT allowed per transaction. The engine refunds what isn't used. */
export const MAX_FEE = 1_500_000n;
