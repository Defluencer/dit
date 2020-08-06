'use strict'

const ipfs = window.IpfsHttpClient({ host: 'localhost', port: 5001, protocol: 'http', })

const topic = "livelike"

var hls

const video = document.getElementById('video')

async function main() {
    if (!ipfs) throw new Error('Connect to a node first')

    await ipfs.pubsub.subscribe(topic, msg => pubsubMessage(msg))

    Hls.DefaultConfig.loader = HlsjsIPFSLoader
    Hls.DefaultConfig.debug = true
    Hls.DefaultConfig.liveDurationInfinity = true
    Hls.DefaultConfig.autoStartLoad = false
    Hls.DefaultConfig.liveSyncDurationCount = 5

    if (Hls.isSupported()) {
        hls = new Hls()

        hls.loadSource('master.m3u8')

        hls.attachMedia(video)
    }
}

var mediaSequence = -5

const streamer = "QmX91oLTbANP7NV5yUYJvWYaRdtfiaLTELbYVX5bA8A9pi"

var playlist = ['#EXTM3U',
    '#EXT-X-VERSION:6',
    '#EXT-X-TARGETDURATION:4',
    '#EXT-X-MEDIA-SEQUENCE:0',
    '#EXT-X-INDEPENDENT-SEGMENTS'];

async function pubsubMessage(msg) {
    const from = msg.from
    const data = msg.data

    console.log(`Message Received from ${from} with data ${data}`)

    if (from !== streamer) return

    //TODO get all variants
    const cid = await dagGet(data, '/1080p60')

    //console.log(`Dag node /1080p60 ${cid}`)

    mediaSequence++

    if (mediaSequence > 0) {
        playlist.splice(5, 2)

        playlist[4] = `#EXT-X-MEDIA-SEQUENCE:${mediaSequence}`
    }

    playlist.push('#EXTINF:4.000,')
    playlist.push(`/${cid}`)

    if (mediaSequence === -4) {

        hls.startLoad()

        video.play()

        console.log('Play video')
    }
}

async function dagGet(cid, path) {
    const result = await ipfs.dag.get(cid, { path })

    return result.value
}

const master = ['#EXTM3U',
    '#EXT-X-VERSION:6',
    '#EXT-X-STREAM-INF:BANDWIDTH=6811200,AVERAGE-BANDWIDTH=6000000,CODECS="avc1.42c02a,mp4a.40.2",RESOLUTION=1920x1080,FRAME-RATE=60.0',
    'livelike/1080p60/index.m3u8',
    '#EXT-X-STREAM-INF:BANDWIDTH=5161200,AVERAGE-BANDWIDTH=4500000,CODECS="avc1.42c020,mp4a.40.2",RESOLUTION=1280x720,FRAME-RATE=60.0',
    'livelike/720p60/index.m3u8',
    '#EXT-X-STREAM-INF:BANDWIDTH=3511200,AVERAGE-BANDWIDTH=3000000,CODECS="avc1.42c01f,mp4a.40.2",RESOLUTION=1280x720,FRAME-RATE=30.0',
    'livelike/720p30/index.m3u8',
    '#EXT-X-STREAM-INF:BANDWIDTH=2411200,AVERAGE-BANDWIDTH=2000000,CODECS="avc1.42c01f,mp4a.40.2",RESOLUTION=854x480,FRAME-RATE=30.0',
    'livelike/480p30/index.m3u8',
    '#EXT-X-INDEPENDENT-SEGMENTS'];

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

        console.log(`Load ${context.url}`)

        const urlParts = context.url.split("/")
        var filename = urlParts[urlParts.length - 1]

        //return data when ask for master playlist
        if (filename === "master.m3u8") {
            let res = master.join('\n')

            console.log(`${res}`)

            const data = (context.responseType === 'arraybuffer') ? str2buf(res) : res
            const response = { url: context.url, data: data }

            callbacks.onSuccess(response, stats, context)

            return;
        }

        //return data when ask for segment playlist
        if (filename === "index.m3u8") {
            //TODO serve all variants

            let res = playlist.join('\n')

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
            console.log('Video segment loaded')

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

    console.log(`Received data for file '${cid}' size: ${value.length}`)

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
