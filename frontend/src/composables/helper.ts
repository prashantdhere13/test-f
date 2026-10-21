import dayjs from 'dayjs'
import utc from 'dayjs/plugin/utc'
import timezone from 'dayjs/plugin/timezone'
dayjs.extend(utc)
dayjs.extend(timezone)

import { useConfig } from '@/stores/config'

// A scheduled "live" block: at its slot the channel switches to the configured
// live-ingest source instead of a media file. It is marked by category / type /
// source / live flag (mirrors the server). Such rows have no media file, so they
// must not be flagged as missing and should be labelled LIVE in the schedule.
const LIVE_ITEM_CATEGORIES = new Set(['live', 'live-feed', 'livefeed', 'ingest', 'srt', 'remote'])

// A "live source" is a network/stream source rather than a local media file:
// srt://, udp://, rtmp(s)://, rtsp://, rist://, http(s)://…, any HLS .m3u8, or
// a path inside the channel's live folder. Such rows are labelled LIVE too.
const LIVE_SOURCE_SCHEMES = [
    'srt://',
    'udp://',
    'rtp://',
    'rtmp://',
    'rtmps://',
    'rtsp://',
    'rist://',
    'http://',
    'https://',
    'multicast://',
]

function looksLikeLiveSource(source: string): boolean {
    const src = source.trim().toLowerCase()
    if (!src) {
        return false
    }
    if (LIVE_ITEM_CATEGORIES.has(src)) {
        return true
    }
    if (src.startsWith('//') || LIVE_SOURCE_SCHEMES.some((scheme) => src.startsWith(scheme))) {
        return true
    }
    if (/\.m3u8(\?|$)/.test(src)) {
        return true
    }
    return src.includes('/live/')
}

export function isLivePlaylistItem(item: PlaylistItem | null | undefined): boolean {
    if (!item) {
        return false
    }

    const meta = item as PlaylistItem & { type?: string; live?: unknown }

    if (String(meta.type || '').trim().toLowerCase() === 'live') {
        return true
    }

    const flag = String(meta.live ?? '').trim().toLowerCase()
    if (meta.live === true || meta.live === 1 || flag === '1' || flag === 'true' || flag === 'yes') {
        return true
    }

    const cat = String(item.category || '').trim().toLowerCase()
    if (LIVE_ITEM_CATEGORIES.has(cat)) {
        return true
    }

    return looksLikeLiveSource(String(item.source || ''))
}

export const stringFormatter = () => {
    function fileSize(bytes: number | undefined, dp = 2) {
        if (!bytes) {
            return 0.0
        }

        const thresh = 1024

        if (Math.abs(bytes) < thresh) {
            return bytes + ' B'
        }

        const units = ['KiB', 'MiB', 'GiB', 'TiB', 'PiB', 'EiB', 'ZiB', 'YiB']
        let u = -1
        const r = 10 ** dp

        do {
            bytes /= thresh
            ++u
        } while (Math.round(Math.abs(bytes) * r) / r >= thresh && u < units.length - 1)

        return bytes.toFixed(dp) + ' ' + units[u]
    }

    function timeToSeconds(time: string): number {
        const t = time.split(':')
        return parseInt(t[0] ?? '0') * 3600 + parseInt(t[1] ?? '0') * 60 + parseInt(t[2] ?? '0')
    }

    function timecodeToSeconds(tc: string): number {
        const t = String(tc ?? '')
            .trim()
            .split(':')
        if (!t[0]) {
            return 0
        }
        const h = parseInt(t[0]) || 0
        const m = parseInt(t[1] ?? '0') || 0
        const s = parseFloat((t[2] ?? '0').replace(',', '.')) || 0
        const f = t[3] !== undefined ? (parseInt(t[3]) || 0) / 100 : 0
        return h * 3600 + m * 60 + s + f
    }

    function secToHMS(sec: number): string {
        const sign = Math.sign(sec)
        sec = Math.abs(sec)

        const hours = Math.floor(sec / 3600)
        sec %= 3600
        const minutes = Math.floor(sec / 60)
        const wholeSeconds = Math.floor(sec % 60)
        const frames = Math.floor(((sec % 60) - wholeSeconds) * 100)

        const m = String(minutes).padStart(2, '0')
        const h = String(hours).padStart(2, '0')
        const s = String(wholeSeconds).padStart(2, '0')
        const f = String(frames).padStart(2, '0')

        const hString = (sign === -1 ? '-' : '') + h

        return `${hString}:${m}:${s}:${f}`
    }

    function secToTime(sec: number): string {
        const sign = Math.sign(sec)
        sec = Math.abs(sec)

        const hours = Math.floor(sec / 3600)
        sec %= 3600
        const minutes = Math.floor(sec / 60)
        const wholeSeconds = Math.floor(sec % 60)

        const m = String(minutes).padStart(2, '0')
        const h = String(hours).padStart(2, '0')
        const s = String(wholeSeconds).padStart(2, '0')

        return (sign === -1 ? '-' : '') + `${h}:${m}:${s}`
    }

    function numberToHex(num: number): string {
        return '0x' + Math.round(num * 255).toString(16)
    }

    function hexToNumber(num: string): number {
        return parseFloat((parseFloat(parseInt(num, 16).toString()) / 255).toFixed(2))
    }

    function filename(path: string): string {
        if (path) {
            const pathArr = path.split(/[\\/]/)
            const name = pathArr[pathArr.length - 1]

            if (name) {
                return name
            } else {
                return path
            }
        } else {
            return ''
        }
    }

    function parent(path: string): string {
        if (path) {
            const pathArr = path.split('/')
            pathArr.pop()

            if (pathArr.length > 0) {
                return pathArr.join('/')
            } else {
                return '/'
            }
        } else {
            return ''
        }
    }

    function toMin(sec: number): string {
        if (sec) {
            const minutes = Math.floor(sec / 60)
            const seconds = Math.round(sec - minutes * 60)
            return `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')} min`
        } else {
            return ''
        }
    }

    function secondsToTime(sec: number) {
        return new Date(sec * 1000 || 0).toISOString().substring(11, 19)
    }

    function mediaType(path: string) {
        const liveType = ['m3u8']
        const videoType = [
            'avi',
            'flv',
            'm2v',
            'm4v',
            'mkv',
            'mov',
            'mp4',
            'mpeg',
            'mpg',
            'mts',
            'mxf',
            'ts',
            'vob',
            'ogv',
            'webm',
            'wmv',
        ]
        const audioType = ['aac', 'aiff', 'flac', 'm4a', 'mp2', 'mp3', 'ogg', 'opus', 'wav', 'wma']
        const imageType = [
            'apng',
            'avif',
            'bmp',
            'exr',
            'gif',
            'jpeg',
            'jpg',
            'png',
            'psd',
            'tga',
            'tif',
            'tiff',
            'webp',
        ]
        const ext = path.split('.').pop()

        if (ext) {
            if (liveType.includes(ext.toLowerCase())) {
                return 'live'
            } else if (videoType.includes(ext.toLowerCase())) {
                return 'video'
            } else if (audioType.includes(ext.toLowerCase())) {
                return 'audio'
            } else if (imageType.includes(ext.toLowerCase())) {
                return 'image'
            }
        }

        return null
    }

    function fileMimeType(filename: string): string {
        const ext = filename.split('.').pop()?.toLowerCase() ?? ''
        const mimeMap: Record<string, string> = {
            ts: 'video/mp2t',
            mts: 'video/mp2t',
            m2v: 'video/mpeg',
            mpeg: 'video/mpeg',
            mpg: 'video/mpeg',
            mp4: 'video/mp4',
            m4v: 'video/mp4',
            mkv: 'video/x-matroska',
            mov: 'video/quicktime',
            avi: 'video/x-msvideo',
            webm: 'video/webm',
            flv: 'video/x-flv',
            wmv: 'video/x-ms-wmv',
            mxf: 'application/mxf',
            vob: 'video/mpeg',
            ogv: 'video/ogg',
            mp3: 'audio/mpeg',
            aac: 'audio/aac',
            flac: 'audio/flac',
            m4a: 'audio/mp4',
            wav: 'audio/wav',
            ogg: 'audio/ogg',
            opus: 'audio/opus',
            m3u8: 'application/x-mpegURL',
        }

        return mimeMap[ext] ?? `video/${ext}`
    }

    function dir_file(path: string): { dir: string; file: string } {
        const index = path.lastIndexOf('/')
        const dir = path.substring(0, index + 1) || '/'
        const file = path.substring(index + 1)

        return { dir, file }
    }

    return {
        fileSize,
        timeToSeconds,
        timecodeToSeconds,
        secToHMS,
        secToTime,
        numberToHex,
        hexToNumber,
        filename,
        parent,
        toMin,
        secondsToTime,
        mediaType,
        fileMimeType,
        dir_file,
    }
}

export const playlistOperations = () => {
    function genUID() {
        return String(Date.now().toString(32) + Math.random().toString(16)).replace(/\./g, '')
    }

    function processPlaylist(date: string, list: PlaylistItem[], forSave: boolean) {
        const configStore = useConfig()

        let begin = configStore.playout.playlist.startInSec

        const newList = []

        for (const item of list) {
            if (configStore.playout.playlist.startInSec === begin) {
                item.date = date
            }

            if (forSave) {
                delete item.date

                if (!item.audio) {
                    delete item.audio
                }

                if (!item.category) {
                    delete item.category
                }

                if (!item.custom_filter) {
                    delete item.custom_filter
                }

                if (!item.scte_marker_type) {
                    delete item.scte_marker_type
                }

                if (item.scte_marker_type !== 'time_signal' || !item.scte_preroll || item.scte_preroll <= 0) {
                    delete item.scte_preroll
                }

                if (!item.secondary_events || !item.secondary_events.length) {
                    delete item.secondary_events
                }

                if (!item.title) {
                    delete item.title
                }

                if (
                    begin + (item.out - item.in) >
                    configStore.playout.playlist.startInSec + configStore.playout.playlist.lengthInSec
                ) {
                    item.out =
                        configStore.playout.playlist.startInSec + configStore.playout.playlist.lengthInSec - begin
                }
            } else {
                if (!item.uid) {
                    item.uid = genUID()
                }

                if (
                    begin >= configStore.playout.playlist.startInSec + configStore.playout.playlist.lengthInSec &&
                    !configStore.playout.playlist.infinit
                ) {
                    item.overtime = true
                } else if (item.overtime) {
                    delete item.overtime
                }

                item.begin = begin
            }

            newList.push(item)

            begin += item.out - item.in
        }

        return newList
    }

    return { processPlaylist, genUID }
}
