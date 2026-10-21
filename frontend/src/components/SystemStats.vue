<template>
    <div class="w-full max-w-[1100px] grid grid-cols-1 xs:grid-cols-2 lg:grid-cols-4 gap-3">
        <div class="col-span-1 xs:col-span-2 lg:col-span-4 card bg-base-100 border border-base-content/10 shadow-sm">
            <div class="card-body !p-5 sm:!p-6">
                <div class="flex flex-wrap items-center justify-between gap-4">
                    <div>
                        <div class="flex flex-wrap items-center gap-x-3 gap-y-1">
                            <span class="text-2xl font-bold tracking-tight">
                                {{ sysStat.system.name }} {{ sysStat.system.version }}
                            </span>
                            <span v-if="sysStat.system.ffp_version" class="badge badge-accent badge-outline gap-1">
                                <i class="bi bi-broadcast" />
                                Zeeplayout v{{ sysStat.system.ffp_version }}
                            </span>
                        </div>
                        <div v-if="sysStat.system.kernel" class="mt-1 text-sm text-base-content/60 font-mono">
                            {{ sysStat.system.kernel }}
                        </div>
                    </div>
                    <span v-if="indexStore.sseConnected" class="badge badge-success badge-sm gap-1.5">
                        <span class="h-2 w-2 rounded-full bg-success animate-pulse" />
                        Live
                    </span>
                </div>
            </div>
        </div>

        <div class="card bg-base-100 border border-base-content/10 shadow-sm">
            <div class="card-body !p-5">
                <div class="flex items-center justify-between">
                    <h2 class="text-base font-bold flex items-center gap-2">
                        <i class="bi bi-cpu text-accent" />
                        {{ t('system.cpu') }}
                    </h2>
                    <span class="text-lg font-bold tabular-nums">{{ sysStat.cpu.usage.toFixed(1) }}%</span>
                </div>
                <progress
                    class="progress w-full mt-2"
                    :class="barClass(sysStat.cpu.usage)"
                    :value="clampPct(sysStat.cpu.usage)"
                    max="100"
                />
                <div class="mt-2 text-sm text-base-content/70">
                    <strong>{{ t('system.cores') }}:</strong> {{ sysStat.cpu.cores }}
                </div>
            </div>
        </div>

        <div class="card bg-base-100 border border-base-content/10 shadow-sm">
            <div class="card-body !p-5">
                <h2 class="text-base font-bold flex items-center gap-2">
                    <i class="bi bi-activity text-accent" />
                    {{ t('system.load') }}
                </h2>
                <div class="mt-2 grid grid-cols-3 gap-2 text-center">
                    <div class="rounded-md bg-base-200 py-2">
                        <div class="text-xs text-base-content/60">1</div>
                        <div class="font-bold tabular-nums">{{ sysStat.load.one }}</div>
                    </div>
                    <div class="rounded-md bg-base-200 py-2">
                        <div class="text-xs text-base-content/60">5</div>
                        <div class="font-bold tabular-nums">{{ sysStat.load.five }}</div>
                    </div>
                    <div class="rounded-md bg-base-200 py-2">
                        <div class="text-xs text-base-content/60">15</div>
                        <div class="font-bold tabular-nums">{{ sysStat.load.fifteen }}</div>
                    </div>
                </div>
            </div>
        </div>

        <div class="card bg-base-100 border border-base-content/10 shadow-sm">
            <div class="card-body !p-5">
                <div class="flex items-center justify-between">
                    <h2 class="text-base font-bold flex items-center gap-2">
                        <i class="bi bi-memory text-accent" />
                        {{ t('system.memory') }}
                    </h2>
                    <span class="text-lg font-bold tabular-nums">{{ memPct.toFixed(1) }}%</span>
                </div>
                <progress class="progress w-full mt-2" :class="barClass(memPct)" :value="memPct" max="100" />
                <div class="mt-2 flex justify-between gap-2 text-sm text-base-content/70">
                    <div>
                        <strong>{{ t('system.usage') }}:</strong> {{ fileSize(sysStat.memory.used) }}
                    </div>
                    <div>
                        <strong>{{ t('system.total') }}:</strong> {{ fileSize(sysStat.memory.total) }}
                    </div>
                </div>
            </div>
        </div>

        <div class="card bg-base-100 border border-base-content/10 shadow-sm">
            <div class="card-body !p-5">
                <div class="flex items-center justify-between">
                    <h2 class="text-base font-bold flex items-center gap-2">
                        <i class="bi bi-hdd text-accent" />
                        {{ t('system.swap') }}
                    </h2>
                    <span class="text-lg font-bold tabular-nums">{{ swapPct.toFixed(1) }}%</span>
                </div>
                <progress class="progress w-full mt-2" :class="barClass(swapPct)" :value="swapPct" max="100" />
                <div class="mt-2 flex justify-between gap-2 text-sm text-base-content/70">
                    <div>
                        <strong>{{ t('system.usage') }}:</strong> {{ fileSize(sysStat.swap.used) }}
                    </div>
                    <div>
                        <strong>{{ t('system.total') }}:</strong> {{ fileSize(sysStat.swap.total) }}
                    </div>
                </div>
            </div>
        </div>

        <div class="card bg-base-100 border border-base-content/10 shadow-sm">
            <div class="card-body !p-5">
                <h2 class="text-base font-bold flex items-center gap-2">
                    <i class="bi bi-hdd-network text-accent" />
                    {{ t('system.network') }}
                    <span v-if="sysStat.network?.name" class="text-xs font-normal text-base-content/60">
                        {{ sysStat.network.name }}
                    </span>
                </h2>
                <div class="mt-2 grid grid-cols-2 gap-2 text-sm">
                    <div class="rounded-md bg-base-200 p-2 flex items-center justify-between gap-2">
                        <span class="flex items-center gap-1 text-base-content/60">
                            <i class="bi bi-arrow-down" />
                            {{ t('system.in') }}
                        </span>
                        <span class="font-bold tabular-nums">{{ fileSize(sysStat.network?.current_in) }}</span>
                    </div>
                    <div class="rounded-md bg-base-200 p-2 flex items-center justify-between gap-2">
                        <span class="flex items-center gap-1 text-base-content/60">
                            <i class="bi bi-arrow-up" />
                            {{ t('system.out') }}
                        </span>
                        <span class="font-bold tabular-nums">{{ fileSize(sysStat.network?.current_out) }}</span>
                    </div>
                    <div class="rounded-md bg-base-200 p-2 flex items-center justify-between gap-2">
                        <span class="text-base-content/60">{{ t('system.in') }}*</span>
                        <span class="font-bold tabular-nums">{{ fileSize(sysStat.network?.total_in) }}</span>
                    </div>
                    <div class="rounded-md bg-base-200 p-2 flex items-center justify-between gap-2">
                        <span class="text-base-content/60">{{ t('system.out') }}*</span>
                        <span class="font-bold tabular-nums">{{ fileSize(sysStat.network?.total_out) }}</span>
                    </div>
                </div>
            </div>
        </div>

        <div v-if="sysStat.storage?.path" class="card bg-base-100 border border-base-content/10 shadow-sm xs:col-span-2">
            <div class="card-body !p-5">
                <div class="flex items-center justify-between">
                    <h2 class="text-base font-bold flex items-center gap-2">
                        <i class="bi bi-stack text-accent" />
                        {{ t('system.storage') }}
                    </h2>
                    <span class="text-lg font-bold tabular-nums">{{ storagePct.toFixed(1) }}%</span>
                </div>
                <progress class="progress w-full mt-2" :class="barClass(storagePct)" :value="storagePct" max="100" />
                <div class="mt-2 flex flex-wrap justify-between gap-2 text-sm text-base-content/70">
                    <div class="truncate font-mono text-xs" :title="sysStat.storage.path">
                        {{ sysStat.storage.path }}
                    </div>
                    <div class="flex gap-4">
                        <span><strong>{{ t('system.used') }}:</strong> {{ fileSize(sysStat.storage.used) }}</span>
                        <span><strong>{{ t('system.size') }}:</strong> {{ fileSize(sysStat.storage.total) }}</span>
                    </div>
                </div>
            </div>
        </div>
        <div v-else class="card bg-base-100 border border-base-content/10 shadow-sm" />
    </div>
</template>
<script setup lang="ts">
import { ref, computed, onBeforeUnmount, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useEventSource } from '@vueuse/core'

import { stringFormatter } from '@/composables/helper'
import { useAuth } from '@/stores/auth'
import { useIndex } from '@/stores/index'
import { useConfig } from '@/stores/config'

const { fileSize } = stringFormatter()
const { t } = useI18n()
const authStore = useAuth()
const configStore = useConfig()
const indexStore = useIndex()

const streamUrl = ref(
    `/data/event/${configStore.channels[configStore.i]?.id}?endpoint=system&uuid=${authStore.uuid}`
)

// 'http://127.0.0.1:8787/data/event/1?endpoint=system&uuid=f2f8c29b-712a-48c5-8919-b535d3a05a3a'
const { status, data, error, close } = useEventSource(streamUrl, [], {
    autoReconnect: {
        retries: -1,
        delay: 1000,
        onFailed() {
            indexStore.sseConnected = false
        },
    },
})

const errorCounter = ref(0)
const defaultStat = {
    cpu: { cores: 1, usage: 0.0 },
    load: { one: 0.0, five: 0.0, fifteen: 0.0 },
    memory: { total: 0.0, used: 0.0, free: 0.0 },
    network: { name: '...', current_in: 0.0, current_out: 0.0, total_in: 0.0, total_out: 0.0 },
    storage: { path: '', total: 0.0, used: 0.0 },
    swap: { total: 0.0, used: 0.0, free: 0.0 },
    system: { name: '...', kernel: '...', version: '...', ffp_version: '...' },
} as SystemStatistics

const sysStat = ref(defaultStat)

function clampPct(value: number): number {
    const n = Number(value)
    if (Number.isNaN(n)) {
        return 0
    }
    return Math.min(100, Math.max(0, n))
}

const memPct = computed(() =>
    sysStat.value.memory.total ? clampPct((sysStat.value.memory.used / sysStat.value.memory.total) * 100) : 0
)
const swapPct = computed(() =>
    sysStat.value.swap.total ? clampPct((sysStat.value.swap.used / sysStat.value.swap.total) * 100) : 0
)
const storagePct = computed(() => {
    const s = sysStat.value.storage
    return s && s.total ? clampPct((s.used / s.total) * 100) : 0
})

function barClass(pct: number): string {
    if (pct >= 85) {
        return 'progress-error'
    }
    if (pct >= 60) {
        return 'progress-warning'
    }
    return 'progress-success'
}

onBeforeUnmount(() => {
    close()
    indexStore.sseConnected = false
})

watch([status, error], async () => {
    if (status.value === 'OPEN') {
        indexStore.sseConnected = true
        errorCounter.value = 0
    } else {
        indexStore.sseConnected = false
        errorCounter.value += 1
        sysStat.value = defaultStat

        if (errorCounter.value > 15) {
            await authStore.obtainUuid()
            streamUrl.value = `/data/event/${configStore.channels[configStore.i]?.id}?endpoint=system&uuid=${
                authStore.uuid
            }`
            errorCounter.value = 0
        }
    }
})

watch([data], () => {
    if (data.value) {
        try {
            sysStat.value = JSON.parse(data.value)
        } catch {
            indexStore.sseConnected = true
        }
    }
})
</script>