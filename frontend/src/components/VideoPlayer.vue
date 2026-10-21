<template>
    <div class="relative">
        <div v-if="options.sources">
            <video
                :id="reference"
                v-show="!useMpegts"
                class="video-js vjs-default-skin vjs-big-play-centered vjs-16-9"
                width="1024"
                height="576"
                crossorigin="anonymous"
            />
            <video
                v-show="useMpegts"
                ref="mpegtsElement"
                class="w-full h-full"
                width="1024"
                height="576"
                controls
                playsinline
                crossorigin="anonymous"
            />
            <AudioMeter v-if="showAudioMeter" :media="meterMedia" />
            <FeedHealthOverlay :media="meterMedia" />
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch } from 'vue'
import videojs from 'video.js'
import 'video.js/dist/video-js.css'
import mpegts from 'mpegts.js'

import AudioMeter from '@/components/AudioMeter.vue'
import FeedHealthOverlay from '@/components/FeedHealthOverlay.vue'

const player = ref<any>()
const mpegtsPlayer = ref<any>(null)
const useMpegts = ref(false)
const mpegtsElement = ref<HTMLVideoElement | null>(null)
const meterMedia = ref<HTMLMediaElement | null>(null)

const props = defineProps({
    options: {
        type: Object,
        required: true,
    },
    reference: {
        type: String,
        required: true,
    },
    showAudioMeter: {
        type: Boolean,
        default: false,
    },
})

function resolveMediaElement() {
    if (useMpegts.value) {
        meterMedia.value = mpegtsElement.value
    } else {
        const el = document.getElementById(props.reference) as HTMLVideoElement | null
        meterMedia.value = el
    }
}

function setupMpegts(type: string, url: string) {
    if (!mpegts.getFeatureList().mseLivePlayback || !mpegtsElement.value) return

    mpegtsPlayer.value = mpegts.createPlayer(
        {
            type,
            isLive: false,
            url,
        },
        {}
    )

    mpegtsPlayer.value.attachMediaElement(mpegtsElement.value)
    mpegtsPlayer.value.load()
    mpegtsPlayer.value.play().catch(() => {})
}

onMounted(() => {
    const sources = (props.options as Record<string, unknown>)?.sources as Array<{ type: string; src: string }> | undefined
    const source = sources?.[0]

    if (source && (source.type === 'video/mp2t' || source.type === 'video/x-flv')) {
        useMpegts.value = true
        const type = source.type === 'video/x-flv' ? 'flv' : 'mpegts'
        setupMpegts(type, source.src)
        resolveMediaElement()
        return
    }

    const volume = localStorage.getItem('volume')

    const mergedOptions = {
        ...(props.options as Record<string, unknown>),
        textTrackSettings: true,
    }

    player.value = videojs(props.reference, mergedOptions, function onPlayerReady() {
        // console.log('onPlayerReady', this);
    })

    resolveMediaElement()

    if (volume !== null) {
        player.value.volume(volume)
    }

    player.value.on('volumechange', () => {
        localStorage.setItem('volume', player.value.volume())
    })

    player.value.on('error', () => {
        // Error logged by video.js; avoid infinite re-render loop
    })

    // Ensure the captions/subtitles button is available and render any
    // closed captions or subtitle tracks (CEA-608/708 CC1, embedded VTT, etc.)
    // provided by the media source. Caption tracks on live/preview TS streams
    // usually only appear after the first segment is parsed, so listen for
    // tracks being added at runtime instead of relying on a single ready shot.
    player.value.ready(() => {
        const playerReady = player.value
        const trackList = playerReady.textTracks && playerReady.textTracks()

        const updateTracks = () => {
            const tracks = playerReady.textTracks && playerReady.textTracks()
            if (!tracks) return
            let hasCaptions = false
            for (const t of Array.from(tracks)) {
                const track = t as { kind?: string; mode?: string; label?: string }
                if (track.kind === 'captions' || track.kind === 'subtitles') {
                    hasCaptions = true
                    if (typeof track.mode === 'string' && track.mode !== 'showing') {
                        track.mode = 'showing'
                    }
                }
            }
            // Expose caption availability to the parent/component consumers.
            playerReady.hasCaptions = hasCaptions
        }

        playerReady.on('loadedmetadata', updateTracks)
        playerReady.on('texttrackchange', updateTracks)

        if (trackList) {
            if (typeof trackList.on === 'function') {
                trackList.on('addtrack', updateTracks)
                trackList.on('removetrack', updateTracks)
            } else if (typeof trackList.addEventListener === 'function') {
                trackList.addEventListener('addtrack', updateTracks)
                trackList.addEventListener('removetrack', updateTracks)
            }
        }

        updateTracks()
    })
})

watch(
    useMpegts,
    () => {
        resolveMediaElement()
    },
    { immediate: true },
)

onBeforeUnmount(() => {
    meterMedia.value = null
    if (mpegtsPlayer.value) {
        mpegtsPlayer.value.unload()
        mpegtsPlayer.value.detachMediaElement()
        mpegtsPlayer.value.destroy()
        mpegtsPlayer.value = null
    }

    if (player.value) {
        player.value.dispose()
    }
})
</script>

<style>
.video-js .vjs-volume-panel.vjs-volume-panel-horizontal {
    width: 10em;
}

.video-js .vjs-volume-panel .vjs-volume-control.vjs-volume-horizontal {
    width: 5em;
    height: 3em;
    margin-right: 0;
    opacity: 1;
}

.video-js .vjs-subs-caps-button {
    display: inline-block;
}

.video-js .vjs-texttrack-settings {
    display: block;
}
</style>