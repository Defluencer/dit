var ipfs
var hls

var getPlaylist

export async function initLibs(topic, pubsubMessage, playlistCallback) {
    if (!Hls.isSupported()) throw new Error('HLS is not supported by your browser!')

    ipfs = await window.IpfsHttpClient({ host: 'localhost', port: 5001, protocol: 'http' })

    await ipfs.pubsub.subscribe(topic, msg => pubsubMessage(msg.from, msg.data))

    getPlaylist = playlistCallback

    Hls.DefaultConfig.loader = HlsjsIPFSLoader
    Hls.DefaultConfig.debug = false
    Hls.DefaultConfig.liveDurationInfinity = true
    Hls.DefaultConfig.autoStartLoad = true
    Hls.DefaultConfig.liveSyncDurationCount = 5

    hls = new Hls()

    hls.on(Hls.Events.MEDIA_ATTACHED, onMediaAttached);
}

function onMediaAttached() {
    hls.loadSource('/livelike/master.m3u8')
}

export function startVideo() {
    const video = document.getElementById('video')

    hls.attachMedia(video)
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

        const url = new URL(context.url);

        const urlParts = url.pathname.split(".")
        var extension = urlParts[urlParts.length - 1]

        //return data when ask for playlist
        if (extension === "m3u8") {
            let res = getPlaylist(context.url)

            const data = (context.responseType === 'text') ? res : str2buf(res)

            stats.loaded = stats.total = data.length
            stats.tload = Math.max(stats.tfirst, performance.now())

            const response = { url: context.url, data: data }

            callbacks.onSuccess(response, stats, context)

            return;
        }

        //return data when ask for video segment
        cat(url.pathname).then(res => {
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

async function cat(path) {
    let value = new Uint8Array(0)

    for await (const buf of ipfs.cat(path)) {
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