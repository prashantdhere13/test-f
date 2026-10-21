<template>
    <div class="h-full">
        <PlayerControl />
        <div
            v-if="showMissingAlert && missingSources.length"
            class="mx-1 mt-1 alert alert-error shadow-sm text-sm"
        >
            <i class="bi-exclamation-triangle-fill shrink-0" />
            <div class="w-full min-w-0">
                <div class="font-bold">
                    {{ t('player.missingAssets') }}
                    <span class="badge badge-error badge-sm ml-1">{{ missingSources.length }}</span>
                </div>
                <ul class="mt-1 max-h-28 overflow-y-auto list-disc list-inside">
                    <li v-for="src in missingSources" :key="src" class="break-all font-mono">
                        {{ src }}
                    </li>
                </ul>
            </div>
            <button class="btn btn-ghost btn-xs shrink-0" :title="t('player.missingRecheck')" @click="recheckMissing">
                <i class="bi-arrow-clockwise" />
            </button>
            <button class="btn btn-ghost btn-xs shrink-0" title="Close" @click="showMissingAlert = false">
                <i class="bi-x-lg" />
            </button>
        </div>
        <div class="flex justify-end p-1">
            <div class="h-[32px] flex">
                <div class="text-warning flex-none flex justify-end p-2">
                    <div
                        v-if="firstLoad && beforeDayStart"
                        class="tooltip tooltip-right tooltip-warning"
                        :data-tip="t('player.dateYesterday')"
                    >
                        <SvgIcon name="warning" />
                    </div>
                </div>
                <VueDatePicker
                    v-if="!configStore.playout.playlist.infinit && configStore.playout.processing.mode !== 'folder'"
                    v-model="listDate"
                    :clearable="false"
                    :hide-navigation="['time']"
                    :action-row="{ showCancel: false, showSelect: false, showPreview: false }"
                    :format="calendarFormat"
                    model-type="yyyy-MM-dd"
                    auto-apply
                    :locale="locale"
                    :dark="indexStore.darkMode"
                    :ui="{ input: 'input input-sm !!w-[300px] text-right !pe-3' }"
                    required
                    @update:model-value=";(beforeDayStart = false), (firstLoad = false)"
                />
            </div>
        </div>
        <div class="p-1 min-h-[260px] h-[calc(100vh-800px)] xl:h-[calc(100vh-480px)]">
            <Splitpanes
                v-if="configStore.playout.processing.mode === 'playlist'"
                class="border border-base-content/30 rounded-sm shadow"
            >
                <Pane
                    v-if="width > 739"
                    class="relative h-full !bg-base-300 rounded-s"
                    min-size="0"
                    max-size="80"
                    size="20"
                >
                    <MediaBrowser :preview="setPreviewData" />
                </Pane>
                <pane>
                    <PlaylistTable ref="playlistTable" :edit-item="editPlaylistItem" :preview="setPreviewData" />
                </pane>
            </Splitpanes>
            <div v-else class="h-full border border-b-2 border-base-content/30 rounded-sm shadow">
                <MediaBrowser :preview="setPreviewData" />
            </div>
        </div>

        <div v-if="configStore.playout.processing.mode === 'playlist'" class="h-16 join flex justify-end p-3">
            <button class="btn btn-sm btn-primary join-item" :title="t('player.copy')" @click="showCopyModal = true">
                <i class="bi-files" />
            </button>
            <button
                v-if="!configStore.playout.playlist.infinit"
                class="btn btn-sm btn-primary join-item"
                :title="t('player.loop')"
                @click="loopClips()"
            >
                <i class="bi-view-stacked" />
            </button>
            <button
                class="btn btn-sm btn-primary join-item"
                :title="t('player.remote')"
                @click="showSourceModal = true"
            >
                <i class="bi-file-earmark-plus" />
            </button>
            <button
                class="btn btn-sm btn-primary join-item"
                :title="t('player.import')"
                @click="showImportModal = true"
            >
                <i class="bi-file-text" />
            </button>
            <button
                class="btn btn-sm btn-primary join-item"
                :title="t('player.generate')"
                @click="mediaStore.getTree('', true), (showPlaylistGenerator = true)"
            >
                <i class="bi-sort-down-alt" />
            </button>
            <button
                class="btn btn-sm btn-primary join-item"
                :title="t('player.reset')"
                @click=";(playlistStore.playlist.length = 0), playlistTable.getPlaylist()"
            >
                <i class="bi-arrow-counterclockwise" />
            </button>
            <button
                class="btn btn-sm btn-primary join-item"
                :title="t('player.save')"
                @click=";(targetDate = listDate), savePlaylist(true)"
            >
                <i class="bi-download" />
            </button>
            <button
                class="btn btn-sm btn-primary join-item"
                :title="t('player.deletePlaylist')"
                @click="showDeleteModal = true"
            >
                <i class="bi-trash" />
            </button>
        </div>

        <GenericModal
            :show="showPreviewModal"
            :title="`${t('media.preview')}: ${previewName}`"
            :hide-buttons="true"
            :modal-action="closePlayer"
        >
            <div class="w-[1024px] max-w-full aspect-video">
                <VideoPlayer v-if="isVideo && previewOpt" reference="previewPlayer" show-audio-meter :options="previewOpt" />                <img v-else :src="previewUrl" class="img-fluid" :alt="previewName" />
            </div>
        </GenericModal>

        <GenericModal :show="showCopyModal" :title="t('player.copyTo')" :modal-action="savePlaylist">
            <VueDatePicker
                v-model="targetDate"
                :clearable="false"
                :hide-navigation="['time']"
                :action-row="{ showCancel: false, showSelect: false, showPreview: false }"
                :format="calendarFormat"
                model-type="yyyy-MM-dd"
                auto-apply
                :locale="locale"
                :dark="indexStore.darkMode"
                :ui="{ input: 'input input-sm !!w-[full text-right !pe-3' }"
                required
            />
        </GenericModal>

        <GenericModal :show="showSourceModal" :title="t('player.addEdit')" :modal-action="processSource">
            <div class="lg:w-96">
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">{{ t('player.title') }}</legend>
                    <input v-model="newSource.title" type="text" name="source" class="input input-sm w-full" />
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">{{ t('player.duration') }}</legend>
                    <TimePicker v-model="newSource.duration" />
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">{{ t('player.in') }}</legend>
                    <TimePicker v-model="newSource.in" />
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">{{ t('player.out') }}</legend>
                    <TimePicker v-model="newSource.out" />
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">{{ t('player.file') }}</legend>
                    <input
                        v-model="newSource.source"
                        type="text"
                        name="source"
                        class="input input-sm w-full"
                        :disabled="newSource.source.includes(configStore.channels[configStore.i]?.storage ?? 'xyz123')"
                    />
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">{{ t('player.audio') }}</legend>
                    <input v-model="newSource.audio" type="text" name="audio" class="input input-sm w-full" />
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">{{ t('player.customFilter') }}</legend>
                    <input
                        v-model="newSource.custom_filter"
                        type="text"
                        name="custom_filter"
                        class="input input-sm w-full"
                    />
                </fieldset>
                <fieldset v-if="isAdslateClip" class="fieldset">
                    <legend class="fieldset-legend">SCTE35 Marker Type</legend>
                    <select
                        v-model="newSource.scte_marker_type"
                        class="select select-sm w-full"
                    >
                        <option value="splice_insert">Splice Insert</option>
                        <option value="time_signal">Time Signal</option>
                    </select>
                </fieldset>
                <fieldset
                    v-if="isAdslateClip && newSource.scte_marker_type === 'time_signal'"
                    class="fieldset"
                >
                    <legend class="fieldset-legend">SCTE35 Pre Roll (seconds)</legend>
                    <input
                        v-model.number="newSource.scte_preroll"
                        type="number"
                        min="0"
                        step="0.1"
                        class="input input-sm w-full"
                    />
                </fieldset>
                <fieldset class="fieldset mt-2 rounded-box w-full">
                    <label class="fieldset-label text-base-content">
                        <input
                            :checked="newSource.category === 'advertisement'"
                            @click="isAd"
                            type="checkbox"
                            class="checkbox"
                        />
                        {{ t('player.ad') }}
                    </label>
                </fieldset>

                <hr class="h-px my-2 bg-base-content/20 border-0" />

                <h4 class="font-bold">Secondary Events (Graphics)</h4>
                <div class="w-full text-xs text-base-content/60">
                    Timed graphic overlays rendered over this clip (NOW/NEXT/LATER lower thirds etc.)
                </div>

                <div
                    v-for="(se, seIdx) in newSource.secondary_events"
                    :key="seIdx"
                    class="w-full border border-base-content/20 rounded-box p-2 mt-2 space-y-1"
                >
                    <div class="flex gap-1 items-center">
                        <select v-model="se.template" class="select select-sm w-40">
                            <option value="NOW">NOW</option>
                            <option value="NEXT">NEXT</option>
                            <option value="LATER">LATER</option>
                            <option value="NNL">NNL</option>
                            <option value="Current-Movie">Current-Movie</option>
                            <option value="custom">Custom</option>
                        </select>
                        <input
                            v-model="se.text"
                            type="text"
                            placeholder="Text"
                            class="input input-sm w-full"
                        />
                        <button
                            class="btn btn-sm btn-error"
                            @click="newSource.secondary_events?.splice(seIdx, 1)"
                        >
                            <i class="bi-x" />
                        </button>
                    </div>
                    <div class="flex gap-2 items-center text-xs">
                        <label class="whitespace-nowrap">In</label>
                        <input
                            :value="secToHMS(se.offset || 0)"
                            type="text"
                            placeholder="00:00:00:00"
                            class="input input-sm w-28 tabular-nums"
                            @change="
                                (e: any) => {
                                    se.offset = Math.max(timecodeToSeconds(e.target.value), 0)
                                    e.target.value = secToHMS(se.offset || 0)
                                }
                            "
                        />
                        <label class="whitespace-nowrap">Duration</label>
                        <input
                            :value="secToHMS(se.duration || 0)"
                            type="text"
                            placeholder="00:00:00:00"
                            class="input input-sm w-28 tabular-nums"
                            @change="
                                (e: any) => {
                                    se.duration = Math.max(timecodeToSeconds(e.target.value), 0)
                                    e.target.value = secToHMS(se.duration || 0)
                                }
                            "
                        />
                        <label class="whitespace-nowrap">Out</label>
                        <span class="tabular-nums font-bold">{{ secToHMS((se.offset || 0) + (se.duration || 0)) }}</span>
                    </div>
                </div>

                <button class="btn btn-sm btn-secondary mt-2" @click="addSecondaryEvent">
                    <i class="bi-plus" /> Add Secondary Event
                </button>

                <hr class="h-px my-2 bg-base-content/20 border-0" />

                <h4 class="font-bold">{{ t('player.splitVideo') }}</h4>

                <fieldset class="fieldset">
                    <legend class="fieldset-legend">{{ t('player.cuts') }}</legend>
                    <input
                        v-model="splitCount"
                        type="number"
                        min="0"
                        step="1"
                        class="input input-sm w-full"
                        @change="splitClip"
                    />
                </fieldset>

                <div v-for="time in splitTimes" :key="time.id" class="form-control mt-2">
                    <TimePicker v-model="time.val" />
                </div>
            </div>
        </GenericModal>

        <GenericModal :show="showImportModal" :title="t('player.import')" :modal-action="importPlaylist">
            <div class="w-full">
                <input
                    type="file"
                    class="file-input file-input-sm w-full"
                    accept=".csv,.xlsx,.json,.m3u,.m3u8,.txt,.asrun"
                    @change="onFileChange"
                />
                <p class="text-xs text-base-content/60 mt-2 whitespace-pre-line">
                    {{ t('player.importHint') }}
                </p>
            </div>
        </GenericModal>

        <GenericModal :show="showDeleteModal" title="Delete Program" :modal-action="deletePlaylist">
            <span>
                {{ t('player.deleteFrom') }} <strong>{{ listDate }}</strong>
            </span>
        </GenericModal>

        <PlaylistGenerator
            v-if="showPlaylistGenerator"
            :close="closeGenerator"
        />

        <div v-if="streamInfo" class="collapse collapse-arrow bg-base-200 mt-2">
            <input type="checkbox" />
            <div class="collapse-title text-sm font-bold">Stream Info</div>
            <div class="collapse-content">
                <div class="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-3 text-sm">
                    <div class="flex flex-col">
                        <span class="text-base-content/60">Status</span>
                        <span :class="streamInfo.active ? 'text-success' : 'text-error'" class="font-mono font-bold">
                            {{ streamInfo.active ? 'ACTIVE' : 'INACTIVE' }}
                        </span>
                    </div>
                    <div class="flex flex-col">
                        <span class="text-base-content/60">Video PID</span>
                        <span class="font-mono">{{ streamInfo.video_pid }}</span>
                    </div>
                    <div class="flex flex-col">
                        <span class="text-base-content/60">Audio PID</span>
                        <span class="font-mono">{{ streamInfo.audio_pid }}</span>
                    </div>
                    <div class="flex flex-col">
                        <span class="text-base-content/60">PMT PID</span>
                        <span class="font-mono">{{ streamInfo.pmt_pid }}</span>
                    </div>
                    <div class="flex flex-col">
                        <span class="text-base-content/60">Service ID</span>
                        <span class="font-mono">{{ streamInfo.service_id }}</span>
                    </div>
                    <div class="flex flex-col">
                        <span class="text-base-content/60">Clip</span>
                        <span class="font-mono">{{ streamInfo.current_index + 1 }} / {{ streamInfo.total_clips }}</span>
                    </div>
                </div>
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
import 'dayjs/locale/de'
import 'dayjs/locale/en'
import 'dayjs/locale/es'
import 'dayjs/locale/pt-br'
import 'dayjs/locale/ru'

import VueDatePicker from '@vuepic/vue-datepicker'
import '@vuepic/vue-datepicker/dist/main.css'

// @ts-ignore
import { Splitpanes, Pane } from 'splitpanes'

import { computed, ref, watch, onBeforeMount, onUnmounted } from 'vue'
import { cloneDeep } from 'lodash-es'
import { useI18n } from 'vue-i18n'
import { useWindowSize } from '@vueuse/core'
import { storeToRefs } from 'pinia'
import { useHead } from '@unhead/vue'

dayjs.extend(customParseFormat)
dayjs.extend(LocalizedFormat)
dayjs.extend(timezone)
dayjs.extend(utc)

import PlayerControl from '@/components/PlayerControl.vue'
import MediaBrowser from '@/components/MediaBrowser.vue'
import PlaylistTable from '@/components/PlaylistTable.vue'
import GenericModal from '@/components/GenericModal.vue'
import PlaylistGenerator from '@/components/PlaylistGenerator.vue'
import VideoPlayer from '@/components/VideoPlayer.vue'
import TimePicker from '@/components/TimePicker.vue'
import SvgIcon from '@/components/SvgIcon.vue'

import { stringFormatter, playlistOperations } from '@/composables/helper'
import { useAuth } from '@/stores/auth'
import { useIndex } from '@/stores/index'
import { useConfig } from '@/stores/config'
import { useMedia } from '@/stores/media'
import { usePlaylist } from '@/stores/playlist'

const { locale, t } = useI18n()
const { width } = useWindowSize({ initialWidth: 800 })
const { mediaType, fileMimeType, secToHMS, timecodeToSeconds } = stringFormatter()
const { processPlaylist, genUID } = playlistOperations()

const authStore = useAuth()
const configStore = useConfig()
const indexStore = useIndex()
const mediaStore = useMedia()
const playlistStore = usePlaylist()

const { listDate, firstLoad, missingSources } = storeToRefs(usePlaylist())
const beforeDayStart = ref(false)
const showMissingAlert = ref(true)
const targetDate = ref(dayjs().tz(configStore.timezone).format('YYYY-MM-DD'))
const playlistTable = ref()
const editId = ref(-1)
const textFile = ref()

let missingTimer: ReturnType<typeof setTimeout> | null = null
let missingInterval: ReturnType<typeof setInterval> | null = null

const sourcesKey = computed(() =>
    playlistStore.playlist
        .map((item) => item.source)
        .filter(Boolean)
        .sort()
        .join('|'),
)

function recheckMissing() {
    showMissingAlert.value = true
    playlistStore.checkMissingSources()
}

watch(sourcesKey, () => {
    showMissingAlert.value = true

    if (missingTimer) {
        clearTimeout(missingTimer)
    }

    missingTimer = setTimeout(() => playlistStore.checkMissingSources(), 250)
})

const showPreviewModal = ref(false)
const showSourceModal = ref(false)
const showImportModal = ref(false)
const showCopyModal = ref(false)
const showDeleteModal = ref(false)
const showPlaylistGenerator = ref(false)

const previewName = ref('')
const previewUrl = ref('')
const previewOpt = ref()
const isVideo = ref(false)
const splitCount = ref(0)
const splitTimes = ref<SplitTime[]>([])

interface StreamInfo {
    active: boolean
    video_pid: number
    audio_pid: number
    pmt_pid: number
    service_id: number
    current_index: number
    elapsed: number
    total_clips: number
    output_param: string
}

const streamInfo = ref<StreamInfo | null>(null)
let streamInfoInterval: ReturnType<typeof setInterval> | null = null

async function fetchStreamInfo() {
    try {
        const channelId = configStore.channels[configStore.i]?.id ?? 1
        const headers = authStore.authHeader as Record<string, string>
        if (!headers?.Authorization) return
        const resp = await fetch(`/api/control/${channelId}/stream-info/`, { headers })
        if (resp.ok) {
            streamInfo.value = await resp.json()
        }
    } catch {
        streamInfo.value = null
    }
}

const newSource = ref({
    begin: 0,
    title: null,
    in: 0,
    out: 0,
    duration: 0,
    category: '',
    custom_filter: '',
    scte_marker_type: 'splice_insert',
    scte_preroll: 0,
    secondary_events: [],
    source: '',
    audio: '',
    uid: '',
} as PlaylistItem)

function addSecondaryEvent() {
    if (!newSource.value.secondary_events) {
        newSource.value.secondary_events = []
    }
    newSource.value.secondary_events.push({
        template: 'NOW',
        offset: 0,
        duration: 20,
        text: '',
    })
}

useHead({
    title: `${t('button.player')}`,
    bodyAttrs: {
        class: computed(() => {
            if (showPlaylistGenerator.value) return 'overflow-hidden'

            return ''
        }),
    },
})

onBeforeMount(() => {
    const currentTime = dayjs().tz(configStore.timezone)

    if (firstLoad.value) {
        if (
            currentTime.format('HH:mm:ss') > '00:00:00' &&
            currentTime.format('HH:mm:ss') < configStore.playout.playlist.day_start
        ) {
            listDate.value = currentTime.subtract(1, 'day').format('YYYY-MM-DD')
            beforeDayStart.value = true
        } else {
            listDate.value = currentTime.format('YYYY-MM-DD')
        }
    }

    if (configStore.onetimeInfo && configStore.playout.playlist.infinit) {
        indexStore.msgAlert('warning', t('player.infinitInfo'), 7)
        configStore.onetimeInfo = false
    }

    fetchStreamInfo()
    streamInfoInterval = setInterval(fetchStreamInfo, 5000)

    playlistStore.checkMissingSources()
    missingInterval = setInterval(() => playlistStore.checkMissingSources(), 30000)
})

onUnmounted(() => {
    if (streamInfoInterval) {
        clearInterval(streamInfoInterval)
        streamInfoInterval = null
    }

    if (missingInterval) {
        clearInterval(missingInterval)
        missingInterval = null
    }

    if (missingTimer) {
        clearTimeout(missingTimer)
        missingTimer = null
    }
})

const calendarFormat = (date: Date) => {
    return dayjs(date).locale(locale.value).format('dddd - LL')
}

function closeGenerator() {
    showPlaylistGenerator.value = false
}

function closePlayer() {
    showPreviewModal.value = false
    isVideo.value = false
}

function setPreviewData(path: string) {
    const storagePath = String(configStore.channels[configStore.i]?.storage ?? '').replace(/\\/g, '/')
    let fullPath = String(path).replace(/\\/g, '/')

    if (!fullPath.includes('/')) {
        // media-browser item: the name is relative to the current folder
        fullPath = `/${mediaStore.folderTree.source}/${fullPath}`.replace(/\/[/]+/g, '/')
    } else if (storagePath && fullPath.toLowerCase().startsWith(`${storagePath}/`.toLowerCase())) {
        // absolute path inside the channel storage -> strip the storage prefix
        fullPath = `/${fullPath.substring(storagePath.length).replace(/^\/+/, '')}`
    } else {
        // older fallback: cut everything up to the stored storage root
        const lastIndex = storagePath.lastIndexOf('/')
        if (lastIndex !== -1) {
            fullPath = fullPath.replace(storagePath.substring(0, lastIndex), '')
        }
    }

    previewName.value = fullPath.split('/').slice(-1)[0] ?? ''
    showPreviewModal.value = true

    if (path.match(/^http/i)) {
        previewUrl.value = path
    } else {
        previewUrl.value = encodeURIComponent(`/file/${configStore.channels[configStore.i]?.id}${fullPath}`).replace(
            /%2F/g,
            '/'
        )
    }

    const ext = previewName.value.split('.').slice(-1)[0]?.toLowerCase()
    const mt = mediaType(previewName.value)

    const fileType = mt === 'live' ? 'application/x-mpegURL' : fileMimeType(previewName.value)

    if (
        mt === 'video' ||
        mt === 'audio' ||
        mt === 'live' ||
        configStore.playout.storage.extensions.includes(`${ext}`)
    ) {
        isVideo.value = true
        previewOpt.value = {
            liveui: false,
            controls: true,
            suppressNotSupportedError: true,
            autoplay: false,
            preload: 'auto',
            sources: [
                {
                    type: fileType,
                    src: previewUrl.value,
                },
            ],
        }
    } else {
        isVideo.value = false
    }
}

function splitClip() {
    splitTimes.value = []
    for (let i = 0; i < splitCount.value; i++) {
        splitTimes.value.push({ id: i, val: (newSource.value.out / (splitCount.value + 1)) * (i + 1) })
    }
}

function splitSource(pos: number) {
    for (let i = 0; i < splitTimes.value.length; i++) {
        const source = cloneDeep(newSource.value)
        source.out = splitTimes.value[i]?.val ?? 0

        if (i > 0) {
            source.in = splitTimes.value[i - 1]?.val ?? 0
        }

        if (pos === -1) {
            playlistStore.playlist.push(source)
        } else if (i === 0) {
            playlistStore.playlist[pos] = source
        } else {
            playlistStore.playlist.splice(pos + i, 0, source)
        }
    }

    const source = cloneDeep(newSource.value)
    source.in = splitTimes.value[splitCount.value - 1]?.val ?? 0

    if (pos === -1) {
        playlistStore.playlist.push(source)
    } else {
        playlistStore.playlist.splice(pos + splitCount.value, 0, source)
    }
}

function processSource(process: boolean) {
    showSourceModal.value = false

    if (process) {
        if (splitCount.value > 0) {
            splitSource(editId.value)
        } else if (editId.value === -1) {
            playlistStore.playlist.push(newSource.value)
        } else {
            playlistStore.playlist[editId.value] = newSource.value
        }

        processPlaylist(listDate.value, playlistStore.playlist, false)
    }

    editId.value = -1
    newSource.value = {
        begin: 0,
        title: '',
        in: 0,
        out: 0,
        duration: 0,
        category: '',
        custom_filter: '',
        scte_marker_type: 'splice_insert',
        scte_preroll: 0,
        secondary_events: [],
        source: '',
        audio: '',
        uid: genUID(),
    }

    splitCount.value = 0
    splitTimes.value = []
}

function editPlaylistItem(i: number) {
    editId.value = i
    showSourceModal.value = true

    newSource.value = {
        begin: playlistStore.playlist[i]?.begin ?? 0,
        title: playlistStore.playlist[i]?.title,
        in: playlistStore.playlist[i]?.in ?? 0,
        out: playlistStore.playlist[i]?.out ?? 0,
        duration: playlistStore.playlist[i]?.duration ?? 0,
        category: playlistStore.playlist[i]?.category,
        custom_filter: playlistStore.playlist[i]?.custom_filter,
        scte_marker_type: playlistStore.playlist[i]?.scte_marker_type ?? 'splice_insert',
        scte_preroll: playlistStore.playlist[i]?.scte_preroll ?? 0,
        secondary_events: playlistStore.playlist[i]?.secondary_events
            ? JSON.parse(JSON.stringify(playlistStore.playlist[i]?.secondary_events))
            : [],
        source: playlistStore.playlist[i]?.source ?? '',
        audio: playlistStore.playlist[i]?.audio,
        uid: playlistStore.playlist[i]?.uid ?? '',
    }
}

const isAdslateClip = computed(() => {
    const name = (newSource.value.source || '').split(/[\\/]+/).pop() ?? ''
    const dot = name.lastIndexOf('.')
    const stem = dot > -1 ? name.slice(0, dot) : name
    const ext = dot > -1 ? name.slice(dot + 1).toLowerCase() : ''

    return (
        stem.toLowerCase().startsWith('adslate') &&
        ['mp4', 'mxf', 'mov', 'mpg', 'mpeg', 'ts'].includes(ext)
    )
})

function isAd(evt: any) {
    if (evt.target.checked) {
        newSource.value.category = 'advertisement'
    } else {
        newSource.value.category = ''
    }
}

function loopClips() {
    const tempList = []
    let length = 0

    while (length < configStore.playlistLength && playlistStore.playlist.length > 0) {
        for (const item of playlistStore.playlist) {
            if (length < configStore.playlistLength) {
                item.uid = genUID()
                tempList.push(cloneDeep(item))
                length += item.out - item.in
            } else {
                break
            }
        }
    }

    playlistStore.playlist = processPlaylist(listDate.value, tempList, false)
}

function onFileChange(evt: any) {
    const files = evt.target.files || evt.dataTransfer.files

    if (!files.length) {
        return
    }

    textFile.value = files
}

async function importPlaylist(imp: boolean) {
    showImportModal.value = false

    if (imp) {
        if (!textFile.value || !textFile.value[0]) {
            return
        }

        const file = textFile.value[0]
        const rawBody = await file.arrayBuffer()

        playlistStore.isLoading = true
        await fetch(
            `/api/file/${configStore.channels[configStore.i]?.id}/import/?file=${encodeURIComponent(
                file.name
            )}&date=${listDate.value}`,
            {
                method: 'PUT',
                headers: { ...authStore.authHeader, 'Content-Type': 'application/octet-stream' },
                body: rawBody,
            }
        )
            .then(async (response) => {
                if (!response.ok) {
                    throw new Error(await response.text())
                }

                return response.json()
            })
            .then(async (response) => {
                indexStore.msgAlert('success', String(response), 2)
                await playlistStore.getPlaylist(listDate.value)
            })
            .catch((e: string) => {
                indexStore.msgAlert('error', e, 4)
            })
    }

    playlistStore.isLoading = false
    textFile.value = null
}

async function savePlaylist(save: boolean) {
    showCopyModal.value = false

    if (save) {
        if (playlistStore.playlist.length === 0) {
            return
        }

        const saveList = processPlaylist(listDate.value, cloneDeep(playlistStore.playlist), true)

        await fetch(`/api/playlist/${configStore.channels[configStore.i]?.id}/`, {
            method: 'POST',
            headers: { ...configStore.contentType, ...authStore.authHeader },
            body: JSON.stringify({
                channel: configStore.channels[configStore.i]?.name,
                date: targetDate.value,
                program: saveList,
            }),
        })
            .then(async (response) => {
                if (!response.ok) {
                    throw new Error(await response.text())
                }

                return response.json()
            })
            .then((response: any) => {
                indexStore.msgAlert('success', response, 2)
            })
            .catch((e: any) => {
                if (e.status === 409) {
                    indexStore.msgAlert('warning', e.data, 2)
                } else {
                    indexStore.msgAlert('error', e, 4)
                }
            })
    }
}

async function deletePlaylist(del: boolean) {
    showDeleteModal.value = false

    if (del) {
        await fetch(`/api/playlist/${configStore.channels[configStore.i]?.id}/${listDate.value}`, {
            method: 'DELETE',
            headers: { ...configStore.contentType, ...authStore.authHeader },
        }).then(() => {
            playlistStore.playlist = []
            indexStore.msgAlert('warning', t('player.deleteSuccess'), 2)
        })
    }
}
</script>
