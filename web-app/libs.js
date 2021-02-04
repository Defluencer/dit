const ipfs = window.IpfsHttpClient({ host: 'localhost', port: 5001, protocol: 'http' })

export async function subscribe(topic, pubsubMessage) {
    await ipfs.pubsub.subscribe(topic, msg => pubsubMessage(msg.from, msg.data))
}

export async function publish(topic, message) {
    await ipfs.pubsub.publish(topic, message)
}

export async function unsubscribe(topic) {
    await ipfs.pubsub.unsubscribe(topic)
}

export async function nameResolve(cid) {
    for await (const path of ipfs.name.resolve(cid)) {
        return path
    }
}

export async function dagGet(cid, path) {
    const result = await ipfs.dag.get(cid, { path })

    return result.value
}

/// Get data from IPFS. Return Uint8Array
export async function cat(path) {
    let value = new Uint8Array(0)

    for await (const buf of ipfs.cat(path)) {
        const newBuf = new Uint8Array(value.length + buf.length)

        newBuf.set(value)
        newBuf.set(buf, value.length)

        value = newBuf
    }

    return value
}

const web3 = new Web3(window.ethereum)

export async function getContenthash(ensName) {
    const result = await web3.eth.ens.getContenthash(ensName);

    return result.decoded
}