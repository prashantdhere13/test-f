<template>
    <div v-if="issues.length" class="feed-health">
        <div class="feed-health__corner" />
        <div class="feed-health__msg">
            <i class="bi-exclamation-triangle-fill mr-1" />
            {{ message }}
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, watch, computed, onBeforeUnmount } from 'vue'
import { useI18n } from 'vue-i18n'

const props = defineProps({
    media: {
        type: Object as () => HTMLMediaElement | null,
        default: null,
    },
})

const { t } = useI18n()

type Issue = 'video' | 'audio' | 'signal'

const issues = ref<Issue[]>([])
let timer: ReturnType<typeof setInterval> | null = null
let lastVideoBytes: number | null = null
let lastAudioBytes: number | null = null
let videoStrikes = 0
let audioStrikes = 0

const message = computed(() => {
    const parts: string[] = []
    if (issues.value.includes('signal')) parts.push(t('player.signalloss'))
    if (issues.value.includes('video') && issues.value.includes('audio')) {
        return parts.concat(t('player.avloss')).join(' / ')
    }
    if (issues.value.includes('video')) parts.push(t('player.videoloss'))
    if (issues.value.includes('audio')) parts.push(t('player.audioloss'))
    return parts.join(' / ')
})

function reset() {
    lastVideoBytes = null
    lastAudioBytes = null
    videoStrikes = 0
    audioStrikes = 0
    issues.value = []
}

function check() {
    const el = props.media
    if (!el) {
        issues.value = []
        return
    }

    const elAny = el as HTMLMediaElement & {
        webkitVideoDecodedByteCount?: number
        webkitAudioDecodedByteCount?: number
    }
    const video = elAny.webkitVideoDecodedByteCount ?? -1
    const audio = elAny.webkitAudioDecodedByteCount ?? -1
    const hasCounters = video >= 0 && audio >= 0

    // user paused playback: that is not a feed problem
    if (el.paused) {
        reset()
        return
    }

    // without OS decoded-byte counters only a total signal loss is visible
    if (!hasCounters) {
        issues.value = el.readyState === 0 ? ['signal'] : []
        return
    }

    if (lastVideoBytes === null || lastAudioBytes === null) {
        lastVideoBytes = video
        lastAudioBytes = audio
        return
    }

    const videoDelta = video - lastVideoBytes
    const audioDelta = audio - lastAudioBytes
    lastVideoBytes = video
    lastAudioBytes = audio

    if (el.readyState === 0) {
        videoStrikes = 0
        audioStrikes = 0
        issues.value = ['signal']
        return
    }

    videoStrikes = videoDelta > 0 ? 0 : videoStrikes + 1
    audioStrikes = audioDelta > 0 ? 0 : audioStrikes + 1

    const next: Issue[] = []
    if (videoStrikes >= 2) next.push('video')
    if (audioStrikes >= 2) next.push('audio')
    issues.value = next
}

watch(
    () => props.media,
    (el, old) => {
        if (old) {
            old.removeEventListener('playing', reset)
            old.removeEventListener('emptied', reset)
        }
        reset()
        if (el) {
            el.addEventListener('playing', reset)
            el.addEventListener('emptied', reset)
        }
    },
    { immediate: true },
)

timer = setInterval(check, 4000)
check()

onBeforeUnmount(() => {
    if (timer) clearInterval(timer)
    const el = props.media
    if (el) {
        el.removeEventListener('playing', reset)
        el.removeEventListener('emptied', reset)
    }
    reset()
})
</script>

<style scoped>
.feed-health {
    position: absolute;
    inset: 0;
    z-index: 5;
    pointer-events: none;
}

.feed-health__corner {
    position: absolute;
    inset: 0;
    box-shadow: inset 0 0 0 4px rgba(239, 68, 68, 0.9);
    animation: feed-pulse 1.2s ease-in-out infinite;
}

.feed-health__corner::after {
    content: '';
    position: absolute;
    top: 0;
    right: 0;
    width: 0;
    height: 0;
    border-style: solid;
    border-width: 0 38px 38px 0;
    border-color: transparent #dc2626 transparent transparent;
}

.feed-health__msg {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 10px 18px;
    background: rgba(0, 0, 0, 0.75);
    color: #f87171;
    border: 1px solid rgba(239, 68, 68, 0.6);
    border-radius: 8px;
    font-weight: 700;
    font-size: 14px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    white-space: nowrap;
}

@keyframes feed-pulse {
    0%,
    100% {
        opacity: 0.5;
    }
    50% {
        opacity: 1;
    }
}
</style>