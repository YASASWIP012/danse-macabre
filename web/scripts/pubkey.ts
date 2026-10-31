// Prints the deployer's public key, which is the burn claim key.
import { readFileSync } from "node:fs";
import { Network, fromHexStr, toHexStr } from "@tari-project/ootle";
import { SecretKeyWallet } from "@tari-project/ootle-secret-key-wallet";
const text = readFileSync("../.env.local", "utf8");
const pick = (name: string) => text.match(new RegExp(`^${name}=(.+)$`, "m"))![1];
const wallet = SecretKeyWallet.fromSecretKey(fromHexStr(pick("DEPLOYER_KEY")), Network.Esmeralda, fromHexStr(pick("DEPLOYER_VIEW")));
console.log(toHexStr(await wallet.getPublicKey()));
