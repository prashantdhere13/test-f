<template>
    <div v-if="barCount" ref="meterRoot" class="audio-meter" :title="t('control.audioMonitorHint')">
        <div class="audio-meter__scale">
            <span v-for="lbl in scaleLabels" :key="lbl.val" :style="{ bottom: lbl.pos + '%' }">{{ lbl.val }}</span>
        </div>
        <div v-for="i in barCount" :key="i" class="audio-meter__meter">
            <div v-for="pos in tickPositions" :key="pos" class="audio-meter__tick" :style="{ bottom: pos + '%' }" />
            <div :ref="(el) => assignBar(i - 1, 'bar', el)" class="audio-meter__fill" />
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, watch, onBeforeUnmount } from 'vue'
import { useI18n } from 'vue-i18n'

import { useIndex } from '@/stores/index'

const indexStore = useIndex()
const { t } = useI18n()

const props = defineProps({
    media: {
        type: Object as () => HTMLMediaElement | null,
        default: null,
    },
})

const barCount = ref(0)
const meterRoot = ref<HTMLElement | null>(null)

interface BarItem {
    bar: HTMLElement | null
    cap: HTMLElement | null
    level: number
    peak: number
}

interface MeterGraph {
    ctx: AudioContext
    analysers: AnalyserNode[]
}

const AudioContextClass: typeof AudioContext | undefined =
    window.AudioContext || (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext

const scaleLabels = [
    { val: '-60', pos: 0 },
    { val: '-50', pos: 17 },
    { val: '-40', pos: 33 },
    { val: '-30', pos: 50 },
    { val: '-20', pos: 67 },
    { val: '-10', pos: 83 },
    { val: '0', pos: 99 },
]
const tickPositions = [0, 17, 33, 50, 67, 83, 99]

const graphCache = new WeakMap<HTMLMediaElement, MeterGraph>()

const barItems: BarItem[] = []
let currentGraph: MeterGraph | null = null
let rafId = 0

function setupGraph(el: HTMLMediaElement): MeterGraph | null {
    if (!AudioContextClass) return null

    const cached = graphCache.get(el)
    if (cached) return cached

    try {
        const ctx = new AudioContextClass()
        const source = ctx.createMediaElementSource(el)
        const channels = Math.min(Math.max(source.channelCount || 1, 1), 8)
        const splitter = ctx.createChannelSplitter(channels)

        source.connect(splitter)

        const analysers: AnalyserNode[] = []
        for (let ch = 0; ch < channels; ch++) {
            const analyser = ctx.createAnalyser()
            analyser.fftSize = 256
            analyser.smoothingTimeConstant = 0.65
            splitter.connect(analyser, ch, 0)
            analysers.push(analyser)
        }

        source.connect(ctx.destination)

        const resume = () => {
            if (ctx.state === 'suspended') ctx.resume().catch(() => {})
        }
        el.addEventListener('play', resume)
        el.addEventListener('loadeddata', resume)
        if (ctx.state === 'suspended') ctx.resume().catch(() => {})

        const graph: MeterGraph = { ctx, analysers }
        graphCache.set(el, graph)
        return graph
    } catch {
        return null
    }
}

function assignBar(index: number, kind: 'bar' | 'cap', el: unknown) {
    if (!barItems[index]) {
        barItems[index] = { bar: null, cap: null, level: 0, peak: 0 }
    }
    barItems[index][kind] = (el as HTMLElement) || null
}

function rebuild() {
    if (!currentGraph) return

    const count = Math.min(indexStore.audioMonitorTracks, currentGraph.analysers.length)
    barItems.length = count
    for (let i = 0; i < count; i++) {
        barItems[i] = { bar: null, cap: null, level: 0, peak: 0 }
    }
    barCount.value = count
}

function stopLoop() {
    if (rafId) {
        cancelAnimationFrame(rafId)
        rafId = 0
    }
    for (const item of barItems) {
        item.level = 0
        item.peak = 0
    }
}

function tick() {
    rafId = requestAnimationFrame(tick)
    const graph = currentGraph
    if (!graph) return

    for (let i = 0; i < barItems.length; i++) {
        const item = barItems[i]
        const analyser = graph.analysers[i]
        if (!item) continue
        let level = 0

        if (analyser) {
            const buffer = new Uint8Array(analyser.fftSize)
            analyser.getByteTimeDomainData(buffer)
            let sum = 0
            for (let j = 0; j < buffer.length; j++) {
                const v = (buffer[j] ?? 0) - 128
                sum += v * v
            }
            const rms = Math.sqrt(sum / buffer.length) / 128
            const db = 20 * Math.log10(rms + 1e-6)
            level = Math.max(0, Math.min(100, ((db + 60) / 60) * 100))
        }

        item.level = level > item.level ? level : Math.max(0, item.level - 0.5)
        item.peak = Math.max(item.level, item.peak - 0.1)

        if (item.bar) item.bar.style.height = `${item.level}%`
        if (item.cap) item.cap.style.height = `${item.peak}%`
    }
}

function start(el: HTMLMediaElement) {
    stopLoop()
    currentGraph = setupGraph(el)

    if (currentGraph) {
        rebuild()
        rafId = requestAnimationFrame(tick)
    } else {
        barCount.value = 0
    }
}

function handleMedia(el: HTMLMediaElement | null) {
    if (!el) {
        stopLoop()
        currentGraph = null
        barCount.value = 0
        return
    }
    start(el)
}

watch(
    () => props.media,
    (el) => handleMedia(el),
    { immediate: true },
)

watch(
    () => props.media?.currentSrc ?? props.media?.src ?? '',
    () => {
        if (props.media && !currentGraph) start(props.media)
    },
)

watch(
    () => indexStore.audioMonitorTracks,
    () => rebuild(),
)

onBeforeUnmount(() => {
    stopLoop()
    currentGraph = null
    barCount.value = 0
    barItems.length = 0
})
</script>

<style scoped>
.audio-meter {
    position: absolute;
    right: 8px;
    bottom: 36px;
    z-index: 2;
    display: flex;
    align-items: flex-end;
    gap: 8px;
    padding: 8px;
    background: rgba(0, 0, 0, 0.45);
    border-radius: 8px;
    pointer-events: none;
}

.audio-meter__scale {
    position: relative;
    height: 300px;
    width: 34px;
    color: #fff;
    font-size: 11px;
}

.audio-meter__scale span {
    position: absolute;
    right: 0;
    transform: translateY(50%);
}

.audio-meter__meter {
    position: relative;
    width: 25px;
    height: 300px;
    background: #111;
    border: 1px solid #555;
    overflow: hidden;
}

.audio-meter__tick {
    position: absolute;
    width: 100%;
    height: 1px;
    background: rgba(255, 255, 255, 0.25);
}

.audio-meter__fill {
    position: absolute;
    bottom: 0;
    width: 100%;
    height: 0%;
    background: linear-gradient(to top, #00ff00 0%, #00ff00 70%, #ffff00 85%, #ff0000 100%);
}
</style>