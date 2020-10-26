export async function initIPFS(topic, pubsubMessage) {
    const ipfs = await window.IpfsHttpClient({ host: 'localhost', port: 5001, protocol: 'http' })

    await ipfs.pubsub.subscribe(topic, msg => pubsubMessage(msg.from, msg.data))
}