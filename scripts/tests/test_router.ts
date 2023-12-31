import {strictEqual} from "assert"
import {Rotosports, Router} from "./lib.js";
import {
    NativeAsset,
    newClient,
    readArtifact,
    TokenAsset,
    NativeSwap,
    RotoSwap
} from "../helpers.js"
import util from "util";
import {Coin } from "@terra-money/terra.js";

async function main() {
    const cl = newClient()
    const network = readArtifact(cl.terra.config.chainID)

    const rotosports = new Rotosports(cl.terra, cl.wallet);
    console.log(`chainID: ${cl.terra.config.chainID} wallet: ${cl.wallet.key.accAddress}`)

    const router = rotosports.router(network.routerAddress);
    console.log("router config: ", await router.queryConfig());

    // 1. Provide ROTO-UST liquidity
    const liquidity_amount = 10000000;
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

    // 4. Assert minimum receive
    await assertMinimumReceive(router, cl.wallet.key.accAddress);

    // 5. Swap tokens
    await swapFromCW20(router, network, rotosports, cl.wallet.key.accAddress);

    // 6. Swap native tokens
    await swapFromNative(router, network, rotosports, cl.wallet.key.accAddress);
}

async function assertMinimumReceive(router: Router, accAddress: string) {
    const swap_amount = 1000;
    try {
        let minReceive = await router.assertMinimumReceive(
            new NativeAsset("uluna", swap_amount.toString()), "1000", "10000000000000000", accAddress);
        console.log("Assert minimum receive: ", util.inspect(minReceive, false, null, true));
    } catch (e: any) {
        console.log("assertMinimumReceive status code: ", e.response.status);
        console.log("assertMinimumReceive data: ", e.response.data);
    }
}

async function swapFromCW20(router: Router, network: any, rotosports: Rotosports, accAddress: string) {
    // to get an error, set the minimum amount to be greater than the exchange amount
    const swap_amount = 1000;
    let min_receive = swap_amount + 1;
    try {
        let resp = await router.swapOperationsCW20(network.tokenAddress, swap_amount.toString(), min_receive.toString(),
            [new RotoSwap(new TokenAsset(network.tokenAddress), new NativeAsset("uusd"))]
        );
        console.log("swap: ", util.inspect(resp, false, null, true));
    } catch (e: any) {
        console.log("swapOperationsCW20 status code: ", e.response.status);
        console.log("swapOperationsCW20 data: ", e.response.data);
    }

    let roto_balance_before_swap = await rotosports.getTokenBalance(network.tokenAddress, accAddress);
    console.log(`roto balance before swap: ${roto_balance_before_swap}`)

    let uluna_balance_before_swap = await rotosports.getNativeBalance(accAddress, "uluna");
    console.log(`uluna balance before swap: ${uluna_balance_before_swap}`)

    // swap with the correct parameters
    try {
        let resp = await router.swapOperationsCW20(network.tokenAddress, swap_amount.toString(), "1",
            [
                new RotoSwap(new TokenAsset(network.tokenAddress), new NativeAsset("uusd")),
                new NativeSwap("uusd", "uluna"),
            ]
        );
        console.log("swap: ", util.inspect(resp, false, null, true));
    } catch (e: any) {
        console.log("swapOperationsCW20 status code: ", e.response.status);
        console.log("swapOperationsCW20 data: ", e.response.data);
    }

    let roto_balance_after_swap = await rotosports.getTokenBalance(network.tokenAddress, accAddress);
    console.log(`roto balance after swap: ${roto_balance_after_swap}`);
    strictEqual(roto_balance_before_swap, roto_balance_after_swap + swap_amount);

    let swapRate = await rotosports.terra.market.swapRate(new Coin("uusd", swap_amount), "uluna");
    console.log("swapRate: ", swapRate);

    let uluna_balance_after_swap = await rotosports.getNativeBalance(accAddress, "uluna");
    console.log(`uluna balance after swap: ${uluna_balance_after_swap}`);

    strictEqual(uluna_balance_before_swap?.amount.toNumber(),
        uluna_balance_after_swap?.add(swapRate).amount.toNumber());
}

async function swapFromNative(router: Router, network: any, rotosports: Rotosports, accAddress: string) {
    const swap_amount = 1000;
    let uluna_balance_before_swap = await rotosports.getNativeBalance(accAddress, "uluna");
    console.log(`uluna balance before swap: ${uluna_balance_before_swap}`);

    let roto_balance_before_swap = await rotosports.getTokenBalance(network.tokenAddress, accAddress);
    console.log(`rotoBalance before swap: ${roto_balance_before_swap}`);

    try {
        let resp = await router.swapOperations([
            new NativeSwap("uluna", "uusd"),
            new RotoSwap(new NativeAsset("uusd"), new TokenAsset(network.tokenAddress)),],
            new Coin("uluna", swap_amount)
        );
        console.log(util.inspect(resp, false, null, true))
    } catch (e: any) {
        console.log("swapOperations status code: ", e.response.status);
        console.log("swapOperations data: ", e.response.data);
    }

    let uluna_balance_after_swap = await rotosports.getNativeBalance(accAddress, "uluna");
    console.log(`uluna balance after swap: ${uluna_balance_after_swap}`);
    strictEqual(uluna_balance_before_swap?.amount.toNumber(), uluna_balance_after_swap?.sub(swap_amount).amount.toNumber());

    let swapRate = await rotosports.terra.market.swapRate(new Coin("uluna", swap_amount), "uusd");
    console.log("swapRate: ", swapRate);

    let roto_balance_after_swap = await rotosports.getTokenBalance(network.tokenAddress, accAddress);
    console.log(`roto balance after swap: ${roto_balance_after_swap}`);

    strictEqual(roto_balance_before_swap, roto_balance_after_swap + swapRate.amount.toNumber());
}

async function provideLiquidity(network: any, rotosports: Rotosports, accAddress: string, poolAddress: string, assets: (NativeAsset|TokenAsset)[]) {
    const pool = rotosports.pair(poolAddress);
    let pair_info = await pool.queryPair();
    console.log(util.inspect(pair_info, false, null, true));

    // Provide liquidity to swap
    await pool.provideLiquidity(assets[0], assets[1])

    let roto_balance = await rotosports.getTokenBalance(network.tokenAddress, accAddress);
    let xroto_balance = await rotosports.getTokenBalance(network.xrotoAddress, accAddress);

    console.log(`ROTO balance: ${roto_balance}`)
    console.log(`xROTO balance: ${xroto_balance}`)
}

main().catch(console.log)
