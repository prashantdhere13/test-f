<template>
    <div class="h-[calc(100vh-140px)] px-2 overflow-auto">
        <div class="max-w-4xl mx-auto py-4">
            <div class="flex items-center gap-3 mb-4">
                <button class="btn btn-sm btn-ghost" :title="t('media.back')" @click="router.push('/media')">
                    <i class="bi-arrow-left" />
                </button>
                <h1 class="text-xl font-bold">{{ asset?.asset_id || 'Asset' }}</h1>
            </div>

            <div v-if="!asset" class="flex items-center justify-center py-16">
                <span class="loading loading-spinner loading-lg" />
            </div>

            <div v-else class="grid grid-cols-1 lg:grid-cols-2 gap-4">
                <div class="card bg-base-200 border border-base-content/20">
                    <div class="card-body">
                        <div v-if="previewType === 'video'" class="aspect-video bg-black rounded-box flex items-center justify-center relative">
                            <video ref="previewAssetVideoRef" :src="previewUrl" controls crossorigin="anonymous" :loop="isLoopingAsset" class="w-full h-full rounded-box">
                                <track
                                    v-if="subtitleUrl"
                                    kind="subtitles"
                                    label="CC 1"
                                    srclang="en"
                                    :src="subtitleUrl"
                                    default
                                />
                            </video>
                            <AudioMeter :media="previewAssetVideoRef" />
                        </div>
                        <div v-else-if="previewType === 'audio'" class="aspect-video bg-black rounded-box flex items-center justify-center">
                            <div class="text-center">
                                <i class="bi-music-note-beamed text-5xl text-base-content/50" />
                                <audio :src="previewUrl" controls class="mt-4 w-72" />
                            </div>
                        </div>
                        <div v-else-if="previewType === 'image'" class="aspect-video bg-black rounded-box flex items-center justify-center">
                            <img :src="previewUrl" :alt="asset.title" class="max-h-full max-w-full object-contain" />
                        </div>
                        <div v-else class="aspect-video bg-black rounded-box flex items-center justify-center">
                            <span class="text-base-content/50">{{ asset.title }}</span>
                        </div>

                        <div class="mt-4 text-sm">
                            <div><strong>Status:</strong> <span class="badge badge-sm" :class="statusClass(asset.status)">{{ asset.status }}</span></div>
                            <div class="mt-2"><strong>Path:</strong> <span class="break-all text-base-content/70">{{ fullPath }}</span></div>
                            <div class="mt-4 flex gap-2">
                                <button
                                    v-if="asset.status !== 'processed'"
                                    class="btn btn-sm btn-primary"
                                    :disabled="isReprocessing"
                                    @click="reprocessAsset"
                                >
                                    <span v-if="isReprocessing" class="loading loading-spinner loading-xs" />
                                    <i v-else class="bi-arrow-repeat" />
                                    {{ isReprocessing ? t('media.processing') : t('media.reprocess') }}
                                </button>
                                <a
                                    class="btn btn-sm"
                                    :href="exportUrl"
                                    :download="asset?.title"
                                    :title="t('media.export')"
                                >
                                    <i class="bi-download" /> {{ t('media.export') }}
                                </a>
                                <button class="btn btn-error btn-sm" @click="showDelete = true">
                                    <i class="bi-trash" /> {{ t('media.delete') }}
                                </button>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="card bg-base-200 border border-base-content/20">
                    <div class="card-body">
                        <h2 class="card-title text-base">Asset Information</h2>
                        <table class="table table-sm">
                            <tbody>
                                <tr>
                                    <td class="text-base-content/60">Title</td>
                                    <td>{{ asset.title }}</td>
                                </tr>
                                <tr>
                                    <td class="text-base-content/60">Asset ID</td>
                                    <td>{{ asset.asset_id }}</td>
                                </tr>
                                <tr>
                                    <td class="text-base-content/60">Category</td>
                                    <td>{{ asset.category }}</td>
                                </tr>
                                <tr>
                                    <td class="text-base-content/60">Type</td>
                                    <td>{{ asset.media_type }}</td>
                                </tr>
                                <tr>
                                    <td class="text-base-content/60">Duration</td>
                                    <td>{{ asset.duration }}</td>
                                </tr>
                                <tr>
                                    <td class="text-base-content/60">Segments</td>
                                    <td>{{ asset.segments }}</td>
                                </tr>
                                <tr>
                                    <td class="text-base-content/60">Audio</td>
                                    <td>{{ asset.audio }}</td>
                                </tr>
                                <tr>
                                    <td class="text-base-content/60">Closed Captions</td>
                                    <td>CC {{ asset.cc ? 1 : 0 }}</td>
                                </tr>
                                <tr>
                                    <td class="text-base-content/60">Frame Rate</td>
                                    <td>{{ asset.frame_rate }}</td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>

        <GenericModal
            :show="showDelete"
            :title="`${t('media.delete')} ${asset?.asset_id}`"
            :text="`${t('media.deleteQuestion')}:<br /><strong>${asset?.path}</strong>`"
            :modal-action="deleteAsset"
        />
    </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter, useRoute } from 'vue-router'

import { useAuth } from '@/stores/auth'
import { useConfig } from '@/stores/config'
import { useIndex } from '@/stores/index'
import { useMedia } from '@/stores/media'

import GenericModal from '@/components/GenericModal.vue'

import AudioMeter from '@/components/AudioMeter.vue'

const { t } = useI18n()
const router = useRouter()
const route = useRoute()
const authStore = useAuth()
const configStore = useConfig()
const indexStore = useIndex()
const mediaStore = useMedia()

const showDelete = ref(false)
const isReprocessing = ref(false)
const previewAssetVideoRef = ref<HTMLVideoElement | null>(null)
const assetPath = ref(decodeURIComponent(String(route.params.path ?? '')))

const fullPath = computed(() => {
    const a = asset.value
    if (!a) return ''
    const storage = configStore.channels[configStore.i]?.storage || ''
    const normalizedStorage = storage.replace(/\\/g, '/')
    return `${normalizedStorage}${a.path}`.replace(/\/+/g, '/')
})

const asset = computed(() => mediaStore.library.find((a) => a.path === assetPath.value))

const previewUrl = computed(() => {
    const id = configStore.channels[configStore.i]?.id
    const a = asset.value
    if (!a) {
        return ''
    }
    const playPath = a.play_path || a.path
    return `/file/${id}${playPath}`
})

const exportUrl = computed(() => {
    const id = configStore.channels[configStore.i]?.id
    const a = asset.value
    if (!a) {
        return ''
    }
    const filePath = a.process_path || a.play_path || a.path
    return `/file/${id}${filePath}`
})

const subtitleUrl = computed(() => {
    const a = asset.value
    if (!a || !a.subtitle) return ''
    const id = configStore.channels[configStore.i]?.id
    const clean = a.subtitle.startsWith('/') ? a.subtitle : `/${a.subtitle}`
    return `/subtitle/${id}${clean}`
})

const isLoopingAsset = computed(() => asset.value?.category === 'logo')

const previewType = computed(() => {
    const a = asset.value
    if (!a) {
        return 'none'
    }
    if (a.category === 'subtitles') {
        return 'none'
    }
    if (a.category === 'audio') {
        return 'audio'
    }
    const ext = (a.title.split('.').slice(-1)[0] || '').toLowerCase()
    if (['png', 'jpg', 'jpeg', 'gif', 'webp', 'bmp'].includes(ext)) {
        return 'image'
    }
    return 'video'
})

document.title = `${asset.value?.asset_id || 'Asset'} | ffplayout`

onMounted(async () => {
    if (!mediaStore.library.length) {
        await mediaStore.getLibrary()
    }
})

watch(
    () => route.params.path,
    () => {
        assetPath.value = decodeURIComponent(String(route.params.path ?? ''))
    }
)

watch(
    () => asset.value?.asset_id,
    (id) => {
        document.title = `${id || 'Asset'} | ffplayout`
    }
)

function statusClass(status: string): string {
    if (status === 'processed') {
        return 'border border-success text-success'
    }
    if (status === 'processing') {
        return 'border border-warning text-warning'
    }
    return 'border border-error text-error'
}

async function deleteAsset(del: boolean) {
    showDelete.value = false

    if (del && asset.value) {
        await fetch(`/api/file/${configStore.channels[configStore.i]?.id}/remove/`, {
            method: 'POST',
            headers: { ...configStore.contentType, ...authStore.authHeader },
            body: JSON.stringify({ source: asset.value.path, recursive: false }),
        })
            .then(async (res) => {
                if (res.status === 200) {
                    indexStore.msgAlert('success', t('media.deleteSuccess'), 3)
                    await mediaStore.getLibrary()
                    router.push('/media')
                } else {
                    indexStore.msgAlert('error', await res.text(), 5)
                }
            })
            .catch((e) => {
                indexStore.msgAlert('error', `${t('media.deleteError')}: ${e}`, 5)
            })
    }
}

async function reprocessAsset() {
    if (!asset.value || isReprocessing.value) return

    isReprocessing.value = true
    const oldPath = asset.value.path
    await mediaStore.reprocessLibrary([oldPath])
    isReprocessing.value = false

    const updated = mediaStore.library.find((a) => a.play_path === oldPath || a.path === oldPath)
    if (updated) {
        assetPath.value = updated.path
    }
}
</script>
