import 'dotenv/config'
import {
    newClient,
    readArtifact,
    queryContractRaw, toDecodedBinary, strToEncodedBinary, getRemoteFile, ARTIFACTS_PATH
} from "./helpers.js"
import {LCDClient} from "@terra-money/terra.js";
import fs from "fs";
import path from "path";

const ROTOSPORTS_CHANGE_LOG_NAME = process.env.ROTOSPORTS_CHANGE_LOG_NAME! || String('core_phoenix')
const ROTOSPORTS_CHANGE_LOG_URL = process.env.ROTOSPORTS_CHANGE_LOG_URL! || String("https://raw.githubusercontent.com/rotosports-fi/rotosports-changelog/main/terra-2/phoenix-1/core_phoenix.json")
const ROTOSPORTS_3RD_PARTY_LOG_NAME = process.env.ROTOSPORTS_CHANGE_LOG_NAME! || String('3rd_party_phoenix')
const ROTOSPORTS_3RD_PARTY_LOG_URL = process.env.ROTOSPORTS_CHANGE_LOG_URL! || String("https://raw.githubusercontent.com/rotosports-fi/rotosports-changelog/main/terra-2/phoenix-1/core_phoenix.json")

interface CInfo {
    address: string,
    localName: string,
    localVersion?: string,
    deployedName: string,
    deployedVersion: string,
}

function buildCInfo(localName: string, address: string, deployedName: string, deployedVersion: string, localVersion?: string): CInfo {
    return {
        address,
        localName,
        localVersion,
        deployedName,
        deployedVersion,
    };
}

async function queryCInfo(terra: LCDClient, name: string, address: string, end_point: string): Promise<CInfo> {
    return await queryContractRaw(terra, end_point)
        .then(resp => {
            let res = JSON.parse(toDecodedBinary(resp.data).toString());
            return buildCInfo(name, address, res.contract, res.version)
        })
        .catch(err => {
            console.log(`${name} - ${address}: ${err}`);
            return buildCInfo("", "", "", "")
        });
}

function changeLogExists(fileName: string, url: string): void {
    try {
        if (!fs.existsSync(path.join(ARTIFACTS_PATH, `${fileName}.json`))) {
            console.log(`File ${fileName} doesn't exists. Start downloading.`)
            getRemoteFile(fileName, url)
            console.log("Finish downloading.")
        }
    } catch(err) {
        console.error(err);
    }
}

async function rotosportsTable(terra: LCDClient) {
    // download config file if does not exists
    changeLogExists(ROTOSPORTS_CHANGE_LOG_NAME, ROTOSPORTS_CHANGE_LOG_URL);
    let network = readArtifact(ROTOSPORTS_CHANGE_LOG_NAME);

    for (const key in network) {
        const value = network[key];
        let end_point = `/cosmwasm/wasm/v1/contract/${value}/raw/${strToEncodedBinary("contract_info")}`;

        // each contract should be saved with `address` substring name in .json config file
        if ( key.includes("address") ){
            await queryCInfo(terra, key, value, end_point).then(resp => {
                if (resp.deployedName.length > 0 ) {
                    console.table(resp);
                }
            })
        }
    }
}

async function rotosports3dPartyTable(terra: LCDClient) {
    // download config file if does not exists
    changeLogExists(ROTOSPORTS_3RD_PARTY_LOG_NAME, ROTOSPORTS_3RD_PARTY_LOG_URL);
    let network = readArtifact("3rd_party_phoenix")

    for (const key in network) {
        const value = network[key];
        let end_point = `/cosmwasm/wasm/v1/contract/${value.address}/raw/${strToEncodedBinary("contract_info")}`;

        await queryCInfo(terra, key, value.address, end_point).then(resp => {
            if (resp.deployedName.length > 0 ) {
                resp.localVersion = value.version
                if (resp.localVersion != resp.deployedVersion ) {
                    console.log("Contract version mismatch!")
                }
                console.table(resp);
            }
        })
    }
}

async function main() {
    const {terra, wallet} = newClient()
    console.log(`chainID: ${terra.config.chainID} wallet: ${wallet.key.accAddress}`)

    const network = readArtifact(terra.config.chainID)
    console.log('Network:', network)

    await rotosportsTable(terra)
    await rotosports3dPartyTable(terra)

}

main().catch(console.log)
