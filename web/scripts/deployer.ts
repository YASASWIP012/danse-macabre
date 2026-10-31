// Creates the deployer wallet once and prints its address. The keys stay in ../.env.local.
import { existsSync, readFileSync, appendFileSync } from "node:fs";
import { Network, toHexStr, fromHexStr } from "@tari-project/ootle";
import { SecretKeyWallet } from "@tari-project/ootle-secret-key-wallet";

const file = "../.env.local";
const text = existsSync(file) ? readFileSync(file, "utf8") : "";
const pick = (name: string) => text.match(new RegExp(`^${name}=(.+)$`, "m"))?.[1];

let wallet: SecretKeyWallet;
if (pick("DEPLOYER_KEY")) {
  wallet = SecretKeyWallet.fromSecretKey(fromHexStr(pick("DEPLOYER_KEY")!), Network.Esmeralda, fromHexStr(pick("DEPLOYER_VIEW")!));
} else {
  wallet = SecretKeyWallet.randomWithViewKey(Network.Esmeralda);
  const raw = wallet as unknown as { ownerSecretKey: Uint8Array };
  appendFileSync(file, `DEPLOYER_KEY=${toHexStr(raw.ownerSecretKey)}\nDEPLOYER_VIEW=${toHexStr(wallet.getViewOnlySecret()!)}\n`);
}
console.log(await wallet.getAddress());
