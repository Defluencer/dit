const web3 = new Web3(window.ethereum)

export async function getContenthash(ensName) {
    const result = await web3.eth.ens.getContenthash(ensName);

    return result.decoded
}