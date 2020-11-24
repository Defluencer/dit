const ipfs = window.IpfsHttpClient({ host: 'localhost', port: 5001, protocol: 'http' })

var getPlaylists

var hls

export async function subscribe(topic, pubsubMessage) {
    await ipfs.pubsub.subscribe(topic, msg => pubsubMessage(msg.from, msg.data))
}

export async function publish(topic, message) {
    await ipfs.pubsub.publish(topic, message)
}

export async function unsubscribe(topic) {
    await ipfs.pubsub.unsubscribe(topic)
}

export function registerPlaylistCallback(callback) {
    getPlaylists = callback
}

export function unregisterPlaylistCallback() {
    getPlaylists = null
}

export function initHLS() {
    if (!Hls.isSupported()) throw new Error('HLS is not supported by your browser!')

    const config = {
        autoStartLoad: false,
        ///startPosition: -1,
        debug: true,
        //capLevelOnFPSDrop: false,
        //capLevelToPlayerSize: false,
        //defaultAudioCodec: undefined,
        initialLiveManifestSize: 1,
        //maxBufferLength: 30,
        //maxMaxBufferLength: 600,
        //maxBufferSize: 60 * 1000 * 1000,
        //maxBufferHole: 0.5,
        //lowBufferWatchdogPeriod: 0.5,
        //highBufferWatchdogPeriod: 3,
        //nudgeOffset: 0.1,
        //nudgeMaxRetry: 3,
        //maxFragLookUpTolerance: 0.25,
        liveSyncDurationCount: 1,
        liveMaxLatencyDurationCount: 5,
        liveDurationInfinity: true,
        liveBackBufferLength: 0,
        //enableWorker: true,
        enableSoftwareAES: false,
        //manifestLoadingTimeOut: 10000,
        //manifestLoadingMaxRetry: 1,
        //manifestLoadingRetryDelay: 1000,
        //manifestLoadingMaxRetryTimeout: 64000,
        //startLevel: undefined,
        //levelLoadingTimeOut: 10000,
        //levelLoadingMaxRetry: 4,
        //levelLoadingRetryDelay: 1000,
        //levelLoadingMaxRetryTimeout: 64000,
        //fragLoadingTimeOut: 20000,
        //fragLoadingMaxRetry: 6,
        //fragLoadingRetryDelay: 1000,
        //fragLoadingMaxRetryTimeout: 64000,
        startFragPrefetch: true,
        //testBandwidth: true,
        //fpsDroppedMonitoringPeriod: 5000,
        //fpsDroppedMonitoringThreshold: 0.2,
        //appendErrorMaxRetry: 3,
        loader: HlsjsIPFSLoader,
        //fLoader: customFragmentLoader,
        //pLoader: customPlaylistLoader,
        //xhrSetup: XMLHttpRequestSetupCallback,
        //fetchSetup: FetchSetupCallback,
        //abrController: AbrController,
        //bufferController: BufferController,
        //capLevelController: CapLevelController,
        //fpsController: FPSController,
        //timelineController: TimelineController,
        enableWebVTT: false,
        enableCEA708Captions: false,
        //stretchShortVideoTrack: false,
        //maxAudioFramesDrift: 1,
        //forceKeyFrameOnDiscontinuity: true,
        //abrEwmaFastLive: 3.0,
        //abrEwmaSlowLive: 9.0,
        //abrEwmaFastVoD: 3.0,
        //abrEwmaSlowVoD: 9.0,
        //abrEwmaDefaultEstimate: 500000,
        //abrBandWidthFactor: 0.95,
        //abrBandWidthUpFactor: 0.7,
        //abrMaxWithRealBitrate: false,
        //maxStarvationDelay: 4,
        //maxLoadingDelay: 4,
        //minAutoBitrate: 0,
        //emeEnabled: false,
        //widevineLicenseUrl: undefined,
        //drmSystemOptions: {},
        //requestMediaKeySystemAccessFunc: requestMediaKeySystemAccess
    }

    hls = new Hls(config)

    /* const video = document.getElementById('video')

    hls.attachMedia(video) */

    /* hls.on(Hls.Events.MEDIA_ATTACHED, () => {
        hls.loadSource('/livelike/master.m3u8')
    }); */

    /* hls.on(Hls.Events.MANIFEST_PARSED, () => {
        hls.startLoad()
    }) */
}

export function attachMedia() {
    const video = document.getElementById('video')

    hls.attachMedia(video)
}

export function loadSource() {
    hls.loadSource('/livelike/master.m3u8')
}

export function startLoad() {
    hls.startLoad()
}

export function destroy() {
    hls.destroy()
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
            let res = getPlaylists(url.pathname)

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