<template>
    <div class="w-full">
        <div class="grid grid-cols-1 md:grid-cols-[auto_512px] xl:grid-cols-[512px_auto_450px]">
            <div class="order-1 p-1">
                <div class="bg-base-100 w-full h-full rounded-sm shadow-sm flex flex-col p-2">
                    <div class="flex items-center justify-between gap-2 pb-1">
                        <span
                            class="text-xs font-bold uppercase tracking-wider text-base-content/50 flex items-center gap-1"
                        >
                            <i class="bi-broadcast" />
                            {{ t('control.livePreview') }}
                        </span>
                        <label
                            class="flex items-center gap-1.5 text-xs text-base-content/60"
                            :title="
                                isAdmin(authStore.role)
                                    ? t('control.audioMonitorHint')
                                    : t('control.audioTracksLockedHint')
                            "
                        >
                            <i class="bi-bar-chart-line text-base leading-none" />
                            {{ t('control.audioTracks') }}
                            <i
                                v-if="!isAdmin(authStore.role)"
                                class="bi-lock-fill text-base leading-none text-warning"
                                :title="t('control.audioTracksLockedHint')"
                            />
                            <select
                                class="select select-xs w-16 tabular-nums"
                                :class="{ 'opacity-60': !isAdmin(authStore.role) }"
                                :value="indexStore.audioMonitorTracks"
                                :disabled="!isAdmin(authStore.role)"
                                @change="indexStore.setAudioMonitorTracks(Number(($event.target as HTMLSelectElement).value))"
                            >
                                <option :value="1">1</option>
                                <option :value="2">2</option>
                            </select>
                        </label>
                    </div>
                    <div class="w-full aspect-video relative">
                        <template v-if="streamExtension === 'flv'">
                            <video
                                ref="httpStreamFlv"
                                class="w-full h-full object-contain"
                                controls
                                playsinline
                                crossorigin="anonymous"
                            />
                            <AudioMeter :media="httpStreamFlv" />
                            <FeedHealthOverlay :media="httpStreamFlv" />
                        </template>
                        <VideoPlayer
                            v-else-if="configStore.showPlayer && configStore.channels[configStore.i]?.preview_url"
                            :key="configStore.i"
                            class="live-player"
                            reference="httpStream"
                            show-audio-meter
                            :options="{
                                liveui: true,
                                controls: true,
                                suppressNotSupportedError: true,
                                autoplay: true,
                                muted: true,
                                playsinline: true,
                                preload: 'auto',
                                playbackRates: [0.5, 1, 1.5, 2],
                                html5: {
                                    vhs: {
                                        overrideNative: true,
                                        enableLowLatencyMode: true,
                                        liveSyncDurationCount: 6,
                                        liveSyncDuration: 15,
                                        liveMaxLatencyDurationCount: 12,
                                        backBufferLength: 30,
                                        forwardBufferLength: 60,
                                        maxLiveSyncPlaybackRate: 2,
                                        fastQualityChange: true,
                                    },
                                    nativeTextTracks: false,
                                    nativeAudioTracks: false,
                                    nativeVideoTracks: false,
                                },
                                sources: [
                                    {
                                        type: 'application/x-mpegURL',
                                        src: configStore.channels[configStore.i]?.preview_url,
                                    },
                                ],
                            }"
                        />
                        <div
                            v-else
                            class="absolute inset-0 flex items-center justify-center text-base-content/40 text-sm"
                        >
                            {{ t('control.noPreview') }}
                        </div>
                    </div>
                </div>
            </div>

            <div
                class="order-3 xl:order-2 col-span-1 md:col-span-2 xl:col-span-1 bg-base-200 h-full grid grid-cols-1 xs:grid-cols-2"
            >
                <div class="col-span-1 p-1">
                    <div
                        class="w-full h-full bg-base-100 rounded-sm flex flex-col items-center justify-center shadow px-6 py-4"
                    >
                        <span class="font-['DigitalNumbers'] text-3xl md:text-2xl 2xl:text-5xl 3xl:text-7xl tracking-tighter">
                            {{ timeStr }}
                        </span>
                        <span class="text-xs font-bold uppercase tracking-wider text-base-content/50 mt-1">{{ t('control.playoutTimezone') }}</span>
                    </div>
                </div>

                <div class="col-span-1 p-1 min-h-[50%]">
                    <div
                        class="w-full h-full bg-base-100 rounded-sm flex flex-col items-center justify-center shadow px-6 py-4"
                    >
                        <span class="font-['DigitalNumbers'] text-3xl md:text-2xl 2xl:text-5xl 3xl:text-7xl tracking-tighter">
                            {{ secToTime(elapsedTime()) }}
                        </span>
                        <span class="text-xs font-bold uppercase tracking-wider text-base-content/50 mt-1">{{ t('control.elapsedTime') }}</span>
                    </div>
                </div>

                <div class="col-span-1 xs:col-span-2 p-1">
                    <div class="w-full h-full bg-base-100 rounded-sm flex items-center px-3 py-2 xl:py-1 shadow">
                        <div class="w-full h-full flex flex-col content-center">
                            <div v-if="playlistStore.ingestRuns" class="h-1/4 font-bold truncate leading-5">
                                {{ t('control.ingest') }}
                            </div>
                            <div
                                v-else
                                class="h-1/4 font-bold text truncate content-center leading-5"
                                :class="{ 'text-base-content/60': playlistStore.current.category === 'advertisement' }"
                                :title="playlistStore.current.title || filename(playlistStore.current.source).replace(/\.[^.]+$/, '')"
                            >
                                {{
                                    playlistStore.current.title ||
                                    filename(playlistStore.current.source).replace(/\.[^.]+$/, '') ||
                                    t('control.noClip')
                                }}
                            </div>
                            <div class="h-1/4 content-center leading-5 text-sm md:text-base">
                                <strong> {{ t('player.duration') }}: </strong>
                                {{ secToHMS(playlistStore.current.duration) }} |
                                <strong> {{ t('player.in') }}: </strong>
                                {{ secToHMS(playlistStore.current.in) }} |
                                <strong> {{ t('player.out') }}: </strong>
                                {{ secToHMS(playlistStore.current.out) }}

                                <template v-if="playlistStore.shift !== 0">
                                    | <strong>{{ t('player.shift') }}:</strong> {{ secToHMS(playlistStore.shift) }}
                                </template>
                            </div>
                            <div class="h-1/4 content-center leading-5 text-sm md:text-base">
                                <strong> {{ t('player.total') }}: </strong>
                                {{
                                    secToHMS(
                                        playlistStore.playlist.reduce(
                                            (total, { in: seek, out }) => total + (out - seek),
                                            0
                                        )
                                    )
                                }}
                            </div>
                            <div class="h-1/4 content-center">
                                <progress
                                    class="progress progress-accent w-full"
                                    :value="
                                        playlistStore.progressValue && playlistStore.progressValue <= 100
                                            ? playlistStore.progressValue
                                            : 0
                                    "
                                    max="100"
                                />
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div class="order-2 xl:order-3 p-1">
                <div class="bg-base-100 h-full flex justify-center rounded-sm shadow">
                    <div class="w-full h-full grid grid-cols-3">
                        <div class="text-center">
                            <div class="w-full h-1/2 aspect-square p-2">
                                <button
                                    :title="t('control.start')"
                                    class="btn btn-primary h-full w-full text-7xl text-lime-600"
                                    :class="playlistStore.playoutIsRunning && 'shadow-glow shadow-lime-600'"
                                    @click="controlProcess('start')"
                                >
                                    <i class="bi-play" />
                                </button>
                            </div>
                            <div class="w-full h-1/2 aspect-square p-2">
                                <button
                                    :title="t('control.last')"
                                    class="btn btn-primary h-full w-full text-7xl text-cyan-600"
                                    @click="controlPlayout('back')"
                                >
                                    <i class="bi-skip-start" />
                                </button>
                            </div>
                        </div>

                        <div class="text-center">
                            <div class="w-full h-1/2 aspect-square p-2">
                                <button
                                    :title="t('control.stop')"
                                    class="btn btn-primary h-full w-full text-7xl text-red-600"
                                    @click="controlProcess('stop')"
                                >
                                    <i class="bi-stop" />
                                </button>
                            </div>

                            <div class="w-full h-1/2 aspect-square p-2">
                                <button
                                    :title="t('control.reset')"
                                    class="btn btn-primary h-full w-full text-6xl text-cyan-600"
                                    @click="controlPlayout('reset')"
                                >
                                    <i class="bi-arrow-repeat" />
                                </button>
                            </div>
                        </div>

                        <div class="text-center">
                            <div class="w-full h-1/2 aspect-square p-2">
                                <button
                                    :title="t('control.restart')"
                                    class="btn btn-primary h-full w-full text-6xl text-yellow-500"
                                    @click="controlProcess('restart')"
                                >
                                    <i class="bi-arrow-clockwise" />
                                </button>
                            </div>

                            <div class="w-full h-1/2 aspect-square p-2">
                                <button
                                    :title="t('control.next')"
                                    class="btn btn-primary h-full w-full text-7xl text-cyan-600"
                                    @click="controlPlayout('next')"
                                >
                                    <i class="bi-skip-end" />
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <div v-if="configStore.playout.output.mode === 'stream'" class="p-1">
            <div class="bg-base-100 rounded-sm shadow-sm p-3">
                <div class="text-lg font-bold mb-2">{{ t('config.streamControl') }}</div>
                <div class="flex flex-wrap gap-3 items-center">
                    <button
                        type="button"
                        class="btn btn-success"
                        :disabled="streamBusy || !configStore.playout.output.output_param"
                        @click="controlStream('start')"
                    >
                        <i class="bi-play-fill mr-1" />
                        {{ t('config.streamStart') }}
                    </button>
                    <button
                        type="button"
                        class="btn btn-error"
                        :disabled="streamBusy"
                        @click="controlStream('stop')"
                    >
                        <i class="bi-stop-fill mr-1" />
                        {{ t('config.streamStop') }}
                    </button>
                    <button
                        type="button"
                        class="btn btn-warning"
                        :disabled="streamBusy"
                        @click="controlStream('restart')"
                    >
                        <i class="bi-arrow-clockwise mr-1" />
                        {{ t('config.streamRestart') }}
                    </button>

                    <span class="divider divider-horizontal mx-1 hidden sm:flex" />

                    <span class="text-xs font-bold uppercase tracking-wider text-base-content/50 flex items-center gap-1 whitespace-nowrap">
                        <i class="bi-aspect-ratio" />
                        {{ t('control.graphicsSection') }}
                    </span>
                    <div class="flex flex-wrap items-center gap-1">
                        <label class="whitespace-nowrap text-sm">{{ t('control.graphicsSelect') }}</label>
                        <select
                            v-model="selectedGraphic"
                            class="select select-sm w-44 tabular-nums"
                            :disabled="controlBusy"
                            :title="t('control.graphicsSelectHint')"
                        >
                            <option value="">
                                {{ t('control.graphicsDefault') }}
                            </option>
                            <option v-for="opt in graphicOptions" :key="opt.path" :value="opt.path">
                                {{ opt.label }}
                            </option>
                        </select>
                        <label class="whitespace-nowrap text-sm">{{ t('control.graphicsControl') }}</label>
                        <input
                            v-model.number="graphicsDuration"
                            type="number"
                            min="1"
                            step="1"
                            class="input input-sm w-24 tabular-nums"
                            :disabled="controlBusy"
                            :title="t('control.graphicsDurationHint')"
                        />
                        <button
                            type="button"
                            class="btn btn-info btn-sm"
                            :disabled="controlBusy || !graphicsDuration || graphicsDuration < 1"
                            @click="enableGraphics"
                        >
                            <i class="bi-play-fill mr-1" />
                            {{ t('control.enableGraphics') }}
                        </button>
                        <button
                            type="button"
                            class="btn btn-neutral btn-sm"
                            :disabled="controlBusy"
                            @click="disableGraphics"
                        >
                            <i class="bi-stop-fill mr-1" />
                            {{ t('control.disableGraphics') }}
                        </button>
                    </div>

                    <span class="divider divider-horizontal mx-1 hidden sm:flex" />

                    <span class="text-xs font-bold uppercase tracking-wider text-base-content/50 flex items-center gap-1 whitespace-nowrap">
                        <i class="bi-broadcast" />
                        {{ t('control.liveSection') }}
                    </span>
                    <div class="flex flex-wrap items-center gap-1">
                        <button
                            type="button"
                            class="btn btn-accent"
                            :disabled="controlBusy || playlistStore.ingestRuns"
                            @click="switchToLive"
                        >
                            <i class="bi-broadcast mr-1" />
                            {{ t('control.live') }}
                        </button>
                        <button
                            type="button"
                            class="btn btn-outline btn-error"
                            :disabled="controlBusy || !playlistStore.ingestRuns"
                            @click="stopLive"
                        >
                            <i class="bi-stop-circle mr-1" />
                            {{ t('control.liveStop') }}
                        </button>
                    </div>
                </div>
                <p class="fieldset-label items-baseline mt-3">{{ t('config.streamControlHint') }}</p>
            </div>
        </div>

        <div v-if="configStore.playout.output.mode === 'hls'" class="p-1">
            <div class="bg-base-100 rounded-sm shadow-sm p-3">
                <div class="text-lg font-bold mb-2">{{ t('config.hlsStreamTitle') }}</div>
                <div class="flex flex-wrap gap-3 items-center">
                    <button
                        type="button"
                        class="btn btn-success"
                        :disabled="streamBusy"
                        @click="controlStream('start')"
                    >
                        <i class="bi-play-fill mr-1" />
                        {{ t('config.streamStart') }}
                    </button>
                    <button
                        type="button"
                        class="btn btn-error"
                        :disabled="streamBusy"
                        @click="controlStream('stop')"
                    >
                        <i class="bi-stop-fill mr-1" />
                        {{ t('config.streamStop') }}
                    </button>
                    <button
                        type="button"
                        class="btn btn-warning"
                        :disabled="streamBusy"
                        @click="controlStream('restart')"
                    >
                        <i class="bi-arrow-clockwise mr-1" />
                        {{ t('config.streamRestart') }}
                    </button>

                    <span class="divider divider-horizontal mx-1 hidden sm:flex" />

                    <span class="text-xs font-bold uppercase tracking-wider text-base-content/50 flex items-center gap-1 whitespace-nowrap">
                        <i class="bi-aspect-ratio" />
                        {{ t('control.graphicsSection') }}
                    </span>
                    <div class="flex flex-wrap items-center gap-1">
                        <label class="whitespace-nowrap text-sm">{{ t('control.graphicsSelect') }}</label>
                        <select
                            v-model="selectedGraphic"
                            class="select select-sm w-44 tabular-nums"
                            :disabled="controlBusy"
                            :title="t('control.graphicsSelectHint')"
                        >
                            <option value="">
                                {{ t('control.graphicsDefault') }}
                            </option>
                            <option v-for="opt in graphicOptions" :key="opt.path" :value="opt.path">
                                {{ opt.label }}
                            </option>
                        </select>
                        <label class="whitespace-nowrap text-sm">{{ t('control.graphicsControl') }}</label>
                        <input
                            v-model.number="graphicsDuration"
                            type="number"
                            min="1"
                            step="1"
                            class="input input-sm w-24 tabular-nums"
                            :disabled="controlBusy"
                            :title="t('control.graphicsDurationHint')"
                        />
                        <button
                            type="button"
                            class="btn btn-info btn-sm"
                            :disabled="controlBusy || !graphicsDuration || graphicsDuration < 1"
                            @click="enableGraphics"
                        >
                            <i class="bi-play-fill mr-1" />
                            {{ t('control.enableGraphics') }}
                        </button>
                        <button
                            type="button"
                            class="btn btn-neutral btn-sm"
                            :disabled="controlBusy"
                            @click="disableGraphics"
                        >
                            <i class="bi-stop-fill mr-1" />
                            {{ t('control.disableGraphics') }}
                        </button>
                    </div>

                    <span class="divider divider-horizontal mx-1 hidden sm:flex" />

                    <span class="text-xs font-bold uppercase tracking-wider text-base-content/50 flex items-center gap-1 whitespace-nowrap">
                        <i class="bi-broadcast" />
                        {{ t('control.liveSection') }}
                    </span>
                    <div class="flex flex-wrap items-center gap-1">
                        <button
                            type="button"
                            class="btn btn-accent"
                            :disabled="controlBusy || playlistStore.ingestRuns"
                            @click="switchToLive"
                        >
                            <i class="bi-broadcast mr-1" />
                            {{ t('control.live') }}
                        </button>
                        <button
                            type="button"
                            class="btn btn-outline btn-error"
                            :disabled="controlBusy || !playlistStore.ingestRuns"
                            @click="stopLive"
                        >
                            <i class="bi-stop-circle mr-1" />
                            {{ t('control.liveStop') }}
                        </button>
                    </div>
                </div>
                <p class="fieldset-label items-baseline mt-3">{{ t('config.streamControlHint') }}</p>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import dayjs from 'dayjs'
import customParseFormat from 'dayjs/plugin/customParseFormat.js'
import LocalizedFormat from 'dayjs/plugin/localizedFormat.js'
import timezone from 'dayjs/plugin/timezone.js'
import utc from 'dayjs/plugin/utc.js'
import mpegts from 'mpegts.js'

import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { throttle } from 'lodash-es'
import { storeToRefs } from 'pinia'
import { useEventSource } from '@vueuse/core'

import { stringFormatter } from '@/composables/helper'
import { isAdmin } from '@/composables/permissions'
import { useAuth } from '@/stores/auth'
import { useIndex } from '@/stores/index'
import { useConfig } from '@/stores/config'
import { useMedia } from '@/stores/media'
import { usePlaylist } from '@/stores/playlist'

import 'dayjs/locale/de'
import 'dayjs/locale/en'
import 'dayjs/locale/es'
import 'dayjs/locale/pt-br'
import 'dayjs/locale/ru'

dayjs.extend(customParseFormat)
dayjs.extend(LocalizedFormat)
dayjs.extend(timezone)
dayjs.extend(utc)

import VideoPlayer from '@/components/VideoPlayer.vue'
import AudioMeter from '@/components/AudioMeter.vue'
import FeedHealthOverlay from '@/components/FeedHealthOverlay.vue'

const { t } = useI18n()
const authStore = useAuth()
const configStore = useConfig()
const indexStore = useIndex()
const playlistStore = usePlaylist()
const mediaStore = useMedia()
    const { filename, secToHMS, secToTime } = stringFormatter()
const { i } = storeToRefs(useConfig())

const currentDefault = {
    uid: '',
    title: t('control.noClip'),
    source: t('control.noClip'),
    duration: 0,
    in: 0,
    out: 0,
} as PlaylistItem

playlistStore.current = currentDefault

const timeStr = ref('00:00:00')
const timer = ref()
const errorCounter = ref(0)
const streamBusy = ref(false)
const controlBusy = ref(false)
const streamExtension = ref(configStore.channels[configStore.i]?.preview_url.split('.').pop())
const httpStreamFlv = ref(null)
const httpFlvSource = ref({
    type: 'flv',
    isLive: true,
    url: configStore.channels[configStore.i]?.preview_url,
})
const mpegtsOptions = ref({
    lazyLoadMaxDuration: 3 * 60,
    liveBufferLatencyChasing: true,
})

const streamUrl = ref(`/data/event/${configStore.channels[configStore.i]?.id}?endpoint=playout&uuid=${authStore.uuid}`)

// 'http://127.0.0.1:8787/data/event/1?endpoint=playout&uuid=f2f8c29b-712a-48c5-8919-b535d3a05a3a'
const { status, data, error, close } = useEventSource(streamUrl, [], {
    autoReconnect: {
        retries: -1,
        delay: 1000,
        onFailed() {
            indexStore.sseConnected = false
        },
    },
})

onMounted(() => {
    let player: any = null

    if (streamExtension.value === 'flv' && mpegts.getFeatureList().mseLivePlayback) {
        if (typeof player !== 'undefined' && player != null) {
            player.unload()
            player.detachMediaElement()
            player.destroy()
            player = null
        }

        player = mpegts.createPlayer(httpFlvSource.value, mpegtsOptions.value)
        player.attachMediaElement(httpStreamFlv.value)
        player.load()
    }

    clock()
    mediaStore.getLibrary()
})

onBeforeUnmount(() => {
    indexStore.sseConnected = false
    close()

    if (timer.value) {
        clearTimeout(timer.value)
    }
})

watch([status, error], async () => {
    if (status.value === 'OPEN') {
        indexStore.sseConnected = true
        errorCounter.value = 0
    } else {
        indexStore.sseConnected = false
        errorCounter.value += 1

        if (errorCounter.value > 11) {
            await authStore.obtainUuid()
            streamUrl.value = `/data/event/${configStore.channels[configStore.i]?.id}?endpoint=playout&uuid=${
                authStore.uuid
            }`
            errorCounter.value = 0
        }
    }
})

watch([data], () => {
    if (data.value) {
        try {
            const playout_status = JSON.parse(data.value)
            playlistStore.setStatus(playout_status)
        } catch {
            indexStore.sseConnected = true
            playlistStore.playoutIsRunning = false
            resetStatus()
        }
    }
})

watch([i], () => {
    resetStatus()

    streamUrl.value = `/data/event/${configStore.channels[configStore.i]?.id}?endpoint=playout&uuid=${authStore.uuid}`

    if (timer.value) {
        clearTimeout(timer.value)
    }
})

function elapsedTime() {
    let elapsed = playlistStore.elapsedSec

    if (elapsed < 0) {
        elapsed = 0
    }

    return elapsed
}

async function clock() {
    async function setTime(resolve: any) {
        timeStr.value = dayjs().tz(configStore.timezone).format('HH:mm:ss')
        timer.value = setTimeout(() => setTime(resolve), 1000)
    }
    return new Promise((resolve) => setTime(resolve))
}

function resetStatus() {
    playlistStore.elapsedSec = 0
    playlistStore.shift = 0
    playlistStore.current = currentDefault
    playlistStore.stopElapsedTicker()
}

const controlProcess = throttle(async (state: string) => {
    /*
        Control playout (start, stop, restart)
    */
    const id = configStore.channels[configStore.i]?.id
    const body: Record<string, any> = { command: state }

    if (state === 'start' || state === 'restart') {
        body.program = playlistStore.playlist.map((item) => ({
            uid: item.uid,
            begin: item.begin,
            title: item.title,
            source: item.source,
            duration: item.duration,
            in: item.in,
            out: item.out,
            secondary_events: item.secondary_events,
        }))
    }

    await fetch(`/api/control/${id}/process/`, {
        method: 'POST',
        headers: { ...configStore.contentType, ...authStore.authHeader },
        body: JSON.stringify(body),
    })
        .then(async (response) => {
            if (state === 'start') {
                playlistStore.scrollToItem = true
                try {
                    const data = await response.json()
                    if (data?.detail) {
                        indexStore.msgAlert('success', String(data.detail), 5)
                    }
                } catch {}
            }
        })
        .catch((e) => {
            indexStore.msgAlert('error', e.data, 3)
        })
}, 2000)

const controlPlayout = throttle(async (state: string) => {
    /*
        Control playout:
        - jump to next clip
        - jump to last clip
        - reset playout state
    */
    const id = configStore.channels[configStore.i]?.id

    await fetch(`/api/control/${id}/playout/`, {
        method: 'POST',
        headers: { ...configStore.contentType, ...authStore.authHeader },
        body: JSON.stringify({ control: state }),
    }).catch((e) => {
        indexStore.msgAlert('error', e.data, 3)
    })
}, 1000)

const controlStream = throttle(async (command: 'start' | 'stop' | 'restart') => {
    /*
        Control streaming output only (start, stop, restart).
    */
    streamBusy.value = true
    try {
        await configStore.controlStream(command)
    } finally {
        streamBusy.value = false
    }
}, 2000)

const controlGraphics = async (action: string, duration?: number, source?: string) => {
    controlBusy.value = true
    const id = configStore.channels[configStore.i]?.id

    try {
        const body: Record<string, any> = { action }
        if (duration !== undefined) {
            body.duration = duration
        }
        if (source) {
            body.source = source
        }

        await fetch(`/api/control/${id}/graphics/`, {
            method: 'POST',
            headers: { ...configStore.contentType, ...authStore.authHeader },
            body: JSON.stringify(body),
        })
            .then(async (response) => {
                if (!response.ok) {
                    throw new Error(await response.text())
                }
                return response.json()
            })
            .then((data) => {
                const msg = typeof data === 'string' ? data : data?.detail || 'ok'
                indexStore.msgAlert('success', String(msg), 3)
            })
            .catch((e) => {
                indexStore.msgAlert('error', e.message || e.data, 3)
            })
    } finally {
        controlBusy.value = false
    }
}

const graphicsDuration = ref(30)

const graphicOptions = computed(() => {
    const lib = mediaStore.library
    if (!lib || !lib.length) {
        return []
    }
    return lib
        .filter((a) => (a.category === 'graphics' || a.category === 'logo') && a.status === 'processed')
        .map((a) => ({
            path: (a.graphic_source || a.process_path || a.path).replace(/^\//, ''),
            label: a.title,
        }))
})

const selectedGraphic = ref('')

const enableGraphics = throttle(async () => {
    const parsed = Number(graphicsDuration.value)
    if (Number.isNaN(parsed) || parsed <= 0) {
        indexStore.msgAlert('warning', t('control.graphicsDurationPrompt'), 3)
        return
    }
    await controlGraphics('enable', parsed, selectedGraphic.value)
}, 2000)

const disableGraphics = throttle(async () => {
    await controlGraphics('disable')
}, 2000)

const switchToLive = throttle(async () => {
    controlBusy.value = true
    const id = configStore.channels[configStore.i]?.id

    try {
        await fetch(`/api/control/${id}/ingest/`, {
            method: 'POST',
            headers: { ...configStore.contentType, ...authStore.authHeader },
            body: JSON.stringify({ action: 'switch' }),
        })
            .then(async (response) => {
                if (!response.ok) {
                    throw new Error(await response.text())
                }
                return response.json()
            })
            .then((data) => {
                const msg = typeof data === 'string' ? data : data?.detail || 'ok'
                indexStore.msgAlert('success', String(msg), 3)
            })
            .catch((e) => {
                indexStore.msgAlert('error', e.message || e.data, 3)
            })
    } finally {
        controlBusy.value = false
    }
}, 2000)

const stopLive = throttle(async () => {
    controlBusy.value = true
    const id = configStore.channels[configStore.i]?.id

    try {
        await fetch(`/api/control/${id}/ingest/`, {
            method: 'POST',
            headers: { ...configStore.contentType, ...authStore.authHeader },
            body: JSON.stringify({ action: 'stop' }),
        })
            .then(async (response) => {
                if (!response.ok) {
                    throw new Error(await response.text())
                }
                return response.json()
            })
            .then((data) => {
                const msg = typeof data === 'string' ? data : data?.message || data?.detail || 'ok'
                indexStore.msgAlert('success', String(msg), 3)
            })
            .catch((e) => {
                indexStore.msgAlert('error', e.message || e.data, 3)
            })
    } finally {
        controlBusy.value = false
    }
}, 2000)
</script>
