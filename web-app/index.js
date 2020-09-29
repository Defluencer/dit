'use strict'

const streamerPeerId = "QmX91oLTbANP7NV5yUYJvWYaRdtfiaLTELbYVX5bA8A9pi"
const gossipsubTopic = "livelike"

const video = document.getElementById('video')

var ipfs
var hls

async function main() {
    if (!Hls.isSupported()) throw new Error('HLS is not supported by your browser!')

    //<script src="https://cdn.jsdelivr.net/npm/ipfs/dist/index.min.js"></script>
    //ipfs = await Ipfs.create({ repo: 'ipfs-' + Math.random() })

    //<script src="https://cdn.jsdelivr.net/npm/ipfs-http-client/dist/index.min.js"></script>
    ipfs = await window.IpfsHttpClient({ host: 'localhost', port: 5001, protocol: 'http' })

    //TODO use ENS name to get IPNS record then get config for the stream

    await ipfs.pubsub.subscribe(gossipsubTopic, msg => pubsubMessage(msg))

    Hls.DefaultConfig.loader = HlsjsIPFSLoader
    Hls.DefaultConfig.debug = false
    Hls.DefaultConfig.liveDurationInfinity = true
    Hls.DefaultConfig.autoStartLoad = false
    //Hls.DefaultConfig.liveSyncDurationCount = 5

    hls = new Hls()

    hls.loadSource('master.m3u8')

    hls.attachMedia(video)
}

//var previousCid = null

async function pubsubMessage(msg) {
    const from = msg.from
    const cid = msg.data

    if (from !== streamerPeerId) return

    console.log(`PubSub reveived => ${cid}`)
    //console.log(`Previous => ${previousCid}`)

    const videoNode = await ipfs.dag.get(cid)

    console.log(`New Node Previous => ${videoNode.value.previous}`)

    updatePlaylists(videoNode)

    //javascript object cannot be equal WTF???
    /* if (liveNode.value.previous == previousCid) {
        console.log("Updating Playlist")

        //console.log(`Variants CID => ${liveNode.value.current}`)

        const variants = await ipfs.dag.get(liveNode.value.current)

        updatePlaylists(variants)
    } else {
        console.log("Rebuilding Playlist")

        rebuildPlaylists(liveNode)
    }

    previousCid = cid */
}

const playlists = [['#EXTM3U',
    '#EXT-X-VERSION:6',
    '#EXT-X-TARGETDURATION:4',
    '#EXT-X-MEDIA-SEQUENCE:0',
    '#EXT-X-INDEPENDENT-SEGMENTS']
    , ['#EXTM3U',
    '#EXT-X-VERSION:6',
    '#EXT-X-TARGETDURATION:4',
    '#EXT-X-MEDIA-SEQUENCE:0',
    '#EXT-X-INDEPENDENT-SEGMENTS']
    , ['#EXTM3U',
    '#EXT-X-VERSION:6',
    '#EXT-X-TARGETDURATION:4',
    '#EXT-X-MEDIA-SEQUENCE:0',
    '#EXT-X-INDEPENDENT-SEGMENTS']
    , ['#EXTM3U',
    '#EXT-X-VERSION:6',
    '#EXT-X-TARGETDURATION:4',
    '#EXT-X-MEDIA-SEQUENCE:0',
    '#EXT-X-INDEPENDENT-SEGMENTS']]

const hlsPlaylistSize = 5
var mediaSequence = -hlsPlaylistSize

function updatePlaylists(variants) {
    mediaSequence++

    if (mediaSequence > 0) {
        playlists[0].splice(5, 2)
        playlists[1].splice(5, 2)
        playlists[2].splice(5, 2)
        playlists[3].splice(5, 2)

        playlists[0][3] = `#EXT-X-MEDIA-SEQUENCE:${mediaSequence}`
        playlists[1][3] = `#EXT-X-MEDIA-SEQUENCE:${mediaSequence}`
        playlists[2][3] = `#EXT-X-MEDIA-SEQUENCE:${mediaSequence}`
        playlists[3][3] = `#EXT-X-MEDIA-SEQUENCE:${mediaSequence}`
    }

    playlists[0].push('#EXTINF:4.000,')
    playlists[0].push(`/${variants.value.quality["1080p60"]}`)

    playlists[1].push('#EXTINF:4.000,')
    playlists[1].push(`/${variants.value.quality["720p60"]}`)

    playlists[2].push('#EXTINF:4.000,')
    playlists[2].push(`/${variants.value.quality["720p30"]}`)

    playlists[3].push('#EXTINF:4.000,')
    playlists[3].push(`/${variants.value.quality["480p30"]}`)

    if (mediaSequence === -4) {
        hls.startLoad()
    }
}

async function rebuildPlaylists(latestVideoNode) {
    const nodes = [latestVideoNode]

    while (nodes[nodes.length - 1].value.previous !== previousCid) {
        const cid = nodes[nodes.length - 1].value.previous

        const videoNode = await ipfs.dag.get(cid)

        if (videoNode.value.previous === null) break //Found first node of the stream, stop here.

        nodes.push(videoNode)

        if (nodes.length >= hlsPlaylistSize) break //Found more node than the list size, stop here.
    }

    nodes.reverse() //Oldest nodes first

    for (const node of nodes) {
        //console.log(`Variants CID => ${node.value.current}`)

        updatePlaylists(node)
    }
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
    '#EXT-X-INDEPENDENT-SEGMENTS']

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
