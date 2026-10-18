import { Connection, Keypair } from "@solana/web3.js";
import { createMint, getOrCreateAssociatedTokenAccount, mintTo, transfer } from "@solana/spl-token";
import * as fs from "fs";
import * as os from "os";
import * as dotenv from "dotenv";

dotenv.config();

const connection = new Connection(process.env.ANCHOR_PROVIDER_URL || "https://api.devnet.solana.com", "confirmed");

const keypairPath = `${os.homedir()}/.config/solana/id.json`;
const secretKey = Uint8Array.from(JSON.parse(fs.readFileSync(keypairPath, "utf8")));
const wallet = Keypair.fromSecretKey(secretKey);

async function main() {
    console.log(`Работаем с кошелька: ${wallet.publicKey.toBase58()}`);

    console.log("1. Создаем токен (Mint)...");
    const mint = await createMint(connection, wallet, wallet.publicKey, null, 9);
    console.log("Mint Address:", mint.toBase58());

    console.log("\n2. Создаем ATA для нашего кошелька...");
    const fromTokenAccount = await getOrCreateAssociatedTokenAccount(connection, wallet, mint, wallet.publicKey);
    console.log("ATA создан:", fromTokenAccount.address.toBase58());

    console.log("\n3. Выпускаем 1000 токенов...");
    const mintTx = await mintTo(connection, wallet, mint, fromTokenAccount.address, wallet, 1000 * 10 ** 9);
    console.log("Хэш транзакции выпуска:", mintTx);

    console.log("\n4. Переводим 100 токенов случайному получателю...");
    const receiver = Keypair.generate();
    const toTokenAccount = await getOrCreateAssociatedTokenAccount(connection, wallet, mint, receiver.publicKey);

    const transferTx = await transfer(
        connection, 
        wallet, 
        fromTokenAccount.address, 
        toTokenAccount.address, 
        wallet.publicKey, 
        100 * 10 ** 9
    );
    console.log("Перевод выполнен! Хэш:", transferTx);
}

main().catch(console.error);