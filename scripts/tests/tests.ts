import {strictEqual} from "assert"
import {Rotosports} from "./lib.js";
import {
    NativeAsset,
    newClient,
    readArtifact,
    TokenAsset,
} from "../helpers.js"


async function main() {
    const { terra, wallet } = newClient()
    const network = readArtifact(terra.config.chainID)

    const rotosports = new Rotosports(terra, wallet);
    console.log(`chainID: ${terra.config.chainID} wallet: ${wallet.key.accAddress}`)

    // 1. Provide liquidity
    await provideLiquidity(network, rotosports, wallet.key.accAddress)

    // 2. Stake ROTO
    await stake(network, rotosports, wallet.key.accAddress)

    // 3. Swap tokens in pool
    await swap(network, rotosports, wallet.key.accAddress)

    // 4. Collect Maker fees
    await collectFees(network, rotosports, wallet.key.accAddress)

    // 5. Withdraw liquidity
    await withdrawLiquidity(network, rotosports, wallet.key.accAddress)

    // 6. Unstake ROTO
    await unstake(network, rotosports, wallet.key.accAddress)
}

async function provideLiquidity(network: any, rotosports: Rotosports, accAddress: string) {
    const liquidity_amount = 100000000;
    const pool_uust_roto = rotosports.pair(network.poolRotoUst);

    // Provide liquidity in order to swap
    await pool_uust_roto.provideLiquidity(new NativeAsset('uusd', liquidity_amount.toString()), new TokenAsset(network.tokenAddress, liquidity_amount.toString()))

    let roto_balance = await rotosports.getTokenBalance(network.tokenAddress, accAddress);
    let xroto_balance = await rotosports.getTokenBalance(network.xrotoAddress, accAddress);

    console.log(`ROTO balance: ${roto_balance}`)
    console.log(`xROTO balance: ${xroto_balance}`)
}

async function withdrawLiquidity(network: any, rotosports: Rotosports, accAddress: string) {
    const pool_uust_roto = rotosports.pair(network.poolRotoUst);

    let pair_info = await pool_uust_roto.queryPair();
    let lp_token_amount = await rotosports.getTokenBalance(pair_info.liquidity_token, accAddress);

    // Withdraw liquidity
    await pool_uust_roto.withdrawLiquidity(pair_info.liquidity_token, lp_token_amount.toString());

    let roto_balance = await rotosports.getTokenBalance(network.tokenAddress, accAddress);
    let xroto_balance = await rotosports.getTokenBalance(network.xrotoAddress, accAddress);

    console.log(`ROTO balance: ${roto_balance}`)
    console.log(`xROTO balance: ${xroto_balance}`)
}

async function stake(network: any, rotosports: Rotosports, accAddress: string) {
    let roto_balance = await rotosports.getTokenBalance(network.tokenAddress, accAddress);
    let xroto_balance = await rotosports.getTokenBalance(network.xrotoAddress, accAddress);

    const staking = rotosports.staking(network.stakingAddress);
    const staking_amount = 100000;

    console.log(`Staking ${staking_amount} ROTO`)
    await staking.stakeRoto(network.tokenAddress, staking_amount.toString())

    let new_roto_balance = await rotosports.getTokenBalance(network.tokenAddress, accAddress);
    let new_xroto_balance = await rotosports.getTokenBalance(network.xrotoAddress, accAddress);

    console.log(`ROTO balance: ${new_roto_balance}`)
    console.log(`xROTO balance: ${new_xroto_balance}`)

    strictEqual(true, new_roto_balance < roto_balance);
    strictEqual(true, new_xroto_balance > xroto_balance);
}

async function unstake(network: any, rotosports: Rotosports, accAddress: string) {
    let roto_balance = await rotosports.getTokenBalance(network.tokenAddress, accAddress);
    let xroto_balance = await rotosports.getTokenBalance(network.xrotoAddress, accAddress);

    const staking = rotosports.staking(network.stakingAddress);

    console.log(`Unstaking ${xroto_balance} xROTO`)
    await staking.unstakeRoto(network.xrotoAddress, xroto_balance.toString())

    let final_roto_balance = await rotosports.getTokenBalance(network.tokenAddress, accAddress);
    let final_xroto_balance = await rotosports.getTokenBalance(network.xrotoAddress, accAddress);

    console.log(`ROTO balance: ${final_roto_balance}`)
    console.log(`xROTO balance: ${final_xroto_balance}`)

    strictEqual(true, final_roto_balance >= roto_balance);
    strictEqual(final_xroto_balance, 0);
}

async function swap(network: any, rotosports: Rotosports, accAddress: string) {
    const pool_uust_roto = rotosports.pair(network.poolRotoUst);
    const factory = rotosports.factory(network.factoryAddress);
    const swap_amount = 10000;

    let pair_info = await pool_uust_roto.queryPair();

    let roto_balance = await rotosports.getTokenBalance(network.tokenAddress, accAddress);
    let xroto_balance = await rotosports.getTokenBalance(network.xrotoAddress, accAddress);

    console.log(`ROTO balance: ${roto_balance}`)
    console.log(`xROTO balance: ${xroto_balance}`)

    let fee_info = await factory.queryFeeInfo('xyk');
    strictEqual(true,  fee_info.fee_address != null, "fee address is not set")
    strictEqual(true,  fee_info.total_fee_bps > 0, "total_fee_bps address is not set")
    strictEqual(true,  fee_info.maker_fee_bps > 0, "maker_fee_bps address is not set")

    console.log('swap some tokens back and forth to accumulate commission')
    for (let index = 0; index < 5; index++) {
        console.log("swap roto to uusd")
        await pool_uust_roto.swapCW20(network.tokenAddress, swap_amount.toString())

        console.log("swap uusd to roto")
        await pool_uust_roto.swapNative(new NativeAsset('uusd', swap_amount.toString()))

        let lp_token_amount = await rotosports.getTokenBalance(pair_info.liquidity_token, accAddress);
        let share_info = await pool_uust_roto.queryShare(lp_token_amount.toString());
        console.log(share_info)
    }
}

async function collectFees(network: any, rotosports: Rotosports, accAddress: string) {
    const maker = rotosports.maker(network.makerAddress);

    let maker_cfg = await maker.queryConfig();
    strictEqual(maker_cfg.roto_token_contract, network.tokenAddress)
    strictEqual(maker_cfg.staking_contract, network.stakingAddress)

    let balances = await maker.queryBalances([new TokenAsset(network.tokenAddress, '0')]);
    strictEqual(true, balances.length > 0, "maker balances are empty. no fees are collected")

    console.log(balances)

    let resp = await maker.collect([network.poolRotoUst])
    console.log(resp)
}

main().catch(console.log)
