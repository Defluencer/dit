const video = document.getElementById('video')

var ipfs
var hls

import { pubsubMessage } from "example";

async function main() {
    if (!Hls.isSupported()) throw new Error('HLS is not supported by your browser!')

    ipfs = await window.IpfsHttpClient({ host: 'localhost', port: 5001, protocol: 'http' })

    await ipfs.pubsub.subscribe("livelike", msg => pubsubMessage(msg))

    Hls.DefaultConfig.loader = HlsjsIPFSLoader
    Hls.DefaultConfig.debug = false
    Hls.DefaultConfig.liveDurationInfinity = true
    Hls.DefaultConfig.autoStartLoad = false
    //Hls.DefaultConfig.liveSyncDurationCount = 5

    hls = new Hls()

    hls.attachMedia(video)

    hls.loadSource('master.m3u8')
}

class HlsjsIPFSLoader {
    constructor(config) {
        if (config.debug === false) {
            this.debug = function () { }
        } else if (config.debug === true) {
            this.debug = console.log
        } else {
            this.debug = config.debug
        }
    }

    load(context, config, callbacks) {
        this.context = context
        this.config = config
        this.callbacks = callbacks
        this.stats = { trequest: performance.now(), retry: 0 }
        this.retryDelay = config.retryDelay
        this.loadInternal()
    }

    loadInternal() {
        const { stats, context, callbacks } = this

        stats.tfirst = Math.max(performance.now(), stats.trequest)
        stats.loaded = 0

        const urlParts = context.url.split("/")
        var filename = urlParts[urlParts.length - 1]

        //return data when ask for master playlist
        if (filename === "master.m3u8") {
            const res = master.join('\n')

            console.log(`${res}`)

            const data = (context.responseType === 'arraybuffer') ? str2buf(res) : res
            const response = { url: context.url, data: data }

            callbacks.onSuccess(response, stats, context)

            return;
        }

        //return data when ask for segment playlist
        if (filename === "index.m3u8") {
            let res

            //use js equivalent of a hash table???
            switch (urlParts[urlParts.length - 2]) {
                case "1080p60":
                    res = playlists[0].join('\n')
                    break;
                case "720p60":
                    res = playlists[1].join('\n')
                    break;
                case "720p30":
                    res = playlists[2].join('\n')
                    break;
                case "480p30":
                    res = playlists[3].join('\n')
                    break;
            }

            console.log(`${res}`)

            const data = (context.responseType === 'arraybuffer') ? str2buf(res) : res

            stats.loaded = stats.total = data.length
            stats.tload = Math.max(stats.tfirst, performance.now())

            const response = { url: context.url, data: data }

            callbacks.onSuccess(response, stats, context)

            return;
        }

        //return data when ask for video segment
        cat(filename).then(res => {
            const data = (context.responseType === 'arraybuffer') ? res : buf2str(res)

            stats.loaded = stats.total = data.length
            stats.tload = Math.max(stats.tfirst, performance.now())

            const response = { url: context.url, data: data }

            callbacks.onSuccess(response, stats, context)
        }, console.error)
    }

    destroy() {
    }

    abort() {
    }
}

async function cat(cid) {
    let value = new Uint8Array(0)

    for await (const buf of ipfs.cat(cid)) {
        const newBuf = new Uint8Array(value.length + buf.length)

        newBuf.set(value)
        newBuf.set(buf, value.length)

        value = newBuf
    }

    return value
}

function buf2str(buf) {
    return String.fromCharCode.apply(null, new Uint8Array(buf))
}

function str2buf(str) {
    var buf = new ArrayBuffer(str.length);

    var bufView = new Uint8Array(buf);

    for (var i = 0, strLen = str.length; i < strLen; i++) {
        bufView[i] = str.charCodeAt(i);
    }

    return buf;
}

main()