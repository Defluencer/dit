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

const quality = '720p30'
const mimeType = 'video/mp4; codecs="avc1.42c01f, mp4a.40.2"'

var mediaSource

var video

var sourceBuffer

var videoCid
var seconds = 0
var minutes = 0
var hours = 0

export function loadVideo(cid) {
    if (!MediaSource.isTypeSupported(mimeType)) {
        console.error('MIME type unsupported!')
        return
    }

    video = document.getElementById('video')

    mediaSource = new MediaSource()

    video.src = URL.createObjectURL(mediaSource)

    videoCid = cid
    seconds = 0
    minutes = 0
    hours = 0

    mediaSource.addEventListener('sourceopen', onVideoReady)
}

async function onVideoReady() {
    sourceBuffer = mediaSource.addSourceBuffer(mimeType)

    sourceBuffer.addEventListener('updateend', onVideoUpdate)

    let path = initSegmentPath()

    cat(path).then(value => {
        sourceBuffer.appendBuffer(value)
    }, () => {
        console.warn('IPFS failed to load initialization segment')
    })
}

async function onVideoUpdate() {
    let path = videoSegmentPath()

    seconds += 4

    if (seconds >= 60) {
        seconds = 0

        minutes += 1
    }

    if (minutes >= 60) {
        minutes = 0

        hours += 1
    }

    cat(path).then(value => {
        sourceBuffer.appendBuffer(value)
    }, () => {
        console.warn('IPFS failed to load next video segment')
        mediaSource.endOfStream()
    })
}

function initSegmentPath() {
    return `${videoCid}/time/hour/${hours}/minute/${minutes}/second/${seconds}/video/init/${quality}`
}

function videoSegmentPath() {
    return `${videoCid}/time/hour/${hours}/minute/${minutes}/second/${seconds}/video/quality/${quality}`
}

var liveTopic
var initialized = false

const utf8Decoder = new TextDecoder("utf-8")

export async function loadStream(topic) {
    if (!MediaSource.isTypeSupported(mimeType)) {
        console.error('MIME type unsupported!')
        return
    }

    video = document.getElementById('video')

    mediaSource = new MediaSource()

    video.src = URL.createObjectURL(mediaSource)

    liveTopic = topic
    initialized = false

    mediaSource.addEventListener('sourceopen', onStreamReady)
}

async function onStreamReady() {
    sourceBuffer = mediaSource.addSourceBuffer(mimeType)

    await ipfs.pubsub.subscribe(liveTopic, msg => onLiveUpdate(msg))
}

function liveVideoSegmentPath(videoCid) {
    return `${videoCid}/quality/${quality}`
}

function liveInitSegmentPath(videoCid) {
    return `${videoCid}/init/${quality}`
}

async function onLiveUpdate(msg) {
    let cid = utf8Decoder.decode(msg.data)

    let path

    if (initialized) {
        path = liveVideoSegmentPath(cid)
    } else {
        path = liveInitSegmentPath(cid)
        initialized = true
    }

    cat(path).then(appendBuffer, () => {
        console.warn('IPFS failed to load video segment')
        mediaSource.endOfStream()
    })
}

async function appendBuffer(data) {
    await until(() => sourceBuffer.updating == false);

    sourceBuffer.appendBuffer(data)
}

function until(condition) {
    return new Promise((resolve) => {
        let interval = setInterval(() => {
            if (!condition()) {
                return
            }

            clearInterval(interval)
            resolve()
        }, 100)
    })
}