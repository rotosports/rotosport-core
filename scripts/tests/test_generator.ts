import {Rotosports, Generator} from "./lib.js";
import {provideLiquidity} from "./test_router.js"
import {
    NativeAsset,
    newClient,
    readArtifact, TokenAsset,
} from "../helpers.js"

async function main() {
    const cl = newClient()
    const network = readArtifact(cl.terra.config.chainID)

    const rotosports = new Rotosports(cl.terra, cl.wallet);
    console.log(`chainID: ${cl.terra.config.chainID} wallet: ${cl.wallet.key.accAddress}`)

    // 1. Provide ROTO-UST liquidity
    const liquidity_amount = 5000000;
    await provideLiquidity(network, rotosports, cl.wallet.key.accAddress, network.poolRotoUst, [
        new NativeAsset('uusd', liquidity_amount.toString()),
        new TokenAsset(network.tokenAddress, liquidity_amount.toString())
    ])

    // 2. Provide LUNA-UST liquidity
    await provideLiquidity(network, rotosports, cl.wallet.key.accAddress, network.poolLunaUst, [
        new NativeAsset('uluna', liquidity_amount.toString()),
        new NativeAsset('uusd', liquidity_amount.toString())
    ])

    // 3. Fetch the pool balances
    let lpTokenRotoUst = await rotosports.getTokenBalance(network.lpTokenRotoUst, cl.wallet.key.accAddress);
    let lpTokenLunaUst = await rotosports.getTokenBalance(network.lpTokenLunaUst, cl.wallet.key.accAddress);

    console.log(`RotoUst balance: ${lpTokenRotoUst}`)
    console.log(`LunaUst balance: ${lpTokenLunaUst}`)

    const generator = rotosports.generator(network.generatorAddress);
    console.log("generator config: ", await generator.queryConfig());

    // 4. Register generators
    await generator.registerGenerator([
        [network.lpTokenRotoUst, "24528"],
        [network.lpTokenLunaUst, "24528"],
    ])

    // 4. Deposit to generator
    await generator.deposit(network.lpTokenRotoUst, "623775")
    await generator.deposit(network.lpTokenLunaUst, "10000000")

    // 5. Fetch the deposit balances
    console.log(`deposited: ${await generator.queryDeposit(network.lpTokenRotoUst, cl.wallet.key.accAddress)}`)
    console.log(`deposited: ${await generator.queryDeposit(network.lpTokenLunaUst, cl.wallet.key.accAddress)}`)

    // 6. Find checkpoint generators limit for user boost
    await findCheckpointGeneratorsLimit(generator, network)
}

async function findCheckpointGeneratorsLimit(generator: Generator, network: any) {
    let generators = []
    for(let i = 0; i < 40; i++) {
        generators.push(network.lpTokenRotoUst)
        generators.push(network.lpTokenLunaUst)
    }

    await generator.checkpointUserBoost(generators)

}

main().catch(console.log)
