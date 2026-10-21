<template>
    <div class="relative w-full h-full !bg-base-300 rounded-e">
        <div v-if="playlistStore.isLoading" class="w-full h-full absolute z-2 flex justify-center bg-base-100/70">
            <span class="loading loading-spinner loading-lg" />
        </div>

        <div
            class="grid bg-base-100 border-b border-base-content/30 rounded-tr-lg py-2 text-sm font-bold text-base-content/70"
            :class="gridClass"
        >
            <div v-if="!configStore.playout.playlist.infinit" class="pl-3 pr-2 text-left">
                {{ t('player.start') }}
            </div>
            <div class="px-3">
                {{ t('player.file') }}
            </div>
            <div class="px-2 hidden 2md:block text-left truncate">
                {{ t('player.title') }}
            </div>
            <div class="px-2 hidden 2md:block text-center">
                {{ t('player.media_type') }}
            </div>
            <div class="px-2 hidden 2md:block text-center">
                {{ t('player.content_type') }}
            </div>
            <div class="px-2 text-center">
                {{ t('player.play') }}
            </div>
            <div class="pl-2 pr-3 hidden 2md:block text-right tabular-nums">
                {{ t('player.duration') }}
            </div>
            <div class="px-2 hidden 2md:block text-right tabular-nums">
                {{ t('player.som') }}
            </div>
            <div class="px-2 hidden 2md:block text-right tabular-nums">
                {{ t('player.eom') }}
            </div>
            <div class="px-2 hidden 2md:block text-center">
                {{ t('player.ad') }}
            </div>
            <div class="px-2 text-center">
                {{ t('player.edit') }}
            </div>
            <div class="pl-2 pr-3 hidden 2md:block text-center">
                {{ t('player.delete') }}
            </div>
        </div>

        <div id="playlist-container" ref="playlistContainer" class="h-[calc(100%-35px)]">
            <VirtualList
                id="sort-container"
                ref="sortContainer"
                v-model="playlistStore.playlist"
                :group="dragGroup"
                class="h-full"
                :handle="'.handle'"
                :data-key="'uid'"
                chosen-class="cursor-grabbing"
                ghost-class="sortable-ghost"
                placeholder-class="sortable-ghost"
                wrap-tag="ul"
                wrap-class="relative list-none text-sm"
                :tableMode="false"
                :animation="100"
                @drop="dropItem"
                @drag="addBG"
            >
                <template #header>
                    <div v-if="playlistStore.playlist.length === 0" class="is-empty relative">
                        <div class="absolute text-8xl text-base-content/40 text-center w-full mt-10">
                            <i class="bi-box-arrow-in-down" />
                        </div>
                    </div>
                </template>
                <template #item="{ record, index }">
                    <li
                        :id="record.uid"
                        :key="record.uid"
                        class="grid odd:bg-base-200 border-b border-base-content/20 duration-500 transition-colors py-2 items-center"
                        :class="[gridClass, rowState(record, index), { 'relative overflow-hidden': isCurrentItem(index) }]"
                    >
                        <div
                            v-if="isCurrentItem(index) && !record.overtime"
                            class="absolute bottom-0 left-0 h-1 bg-green-500 transition-[width] ease-linear duration-1000"
                            :style="{ width: currentProgress + '%' }"
                        />
                        <div v-if="!configStore.playout.playlist.infinit" class="pl-3 pr-2 text-left tabular-nums whitespace-nowrap">
                            {{ secondsToTime(record.begin) }}
                        </div>
                        <div class="px-3 text-left truncate" :class="{ 'handle cursor-grab': width > 768 }" :title="nameTitle(record)">
                            {{ (record.asset_id || filename(record.source)).replace(/\.[^.]+$/, '') }}
                            <span
                                v-if="isLiveItem(record)"
                                class="inline-flex items-center gap-1 ml-1 px-1.5 rounded-full bg-red-500/30 text-red-100 text-xs align-middle font-bold"
                                title="Scheduled live block (switches to the live ingest source)"
                            >
                                <i class="bi-broadcast" />
                                LIVE
                            </span>
                            <span
                                v-if="record.segment && record.segment_count && record.segment_count > 1"
                                class="inline-flex items-center gap-1 ml-1 px-1.5 rounded-full bg-primary/25 text-primary text-xs align-middle font-normal"
                                :title="`Segment ${record.segment} of ${record.segment_count} (${secToHMS(record.in)} - ${secToHMS(record.out)})`"
                            >
                                {{ record.segment }}/{{ record.segment_count }}
                            </span>
                            <span
                                v-if="record.secondary_events?.length"
                                class="inline-flex items-center gap-1 ml-1 px-1.5 rounded-full bg-violet-500/25 text-violet-200 text-xs align-middle"
                                :title="seTooltip(record)"
                            >
                                <i class="bi-layers" />
                                {{ record.secondary_events.length }}
                            </span>
                        </div>
                        <div class="px-2 hidden 2md:block text-left truncate" :title="record.title || ''">
                            {{ record.title || '' }}
                        </div>
                        <div class="px-2 hidden 2md:block text-center text-xs" :class="{ 'text-red-300 font-bold': isLiveItem(record) }">
                            {{ isLiveItem(record) ? 'Live' : record.category === 'advertisement' ? 'Secondary' : 'Primary' }}
                        </div>
                        <div class="px-2 hidden 2md:block text-center text-xs">
                            <span v-if="isLiveItem(record)" class="text-red-300 font-bold">live</span>
                            <span v-else>{{ record.category === 'advertisement' ? 'adslate' : (record.secondary_events?.length ? 'graphics' : 'media') }}</span>
                        </div>
                        <div class="text-center hover:text-base-content/70">
                            <button class="cursor-pointer" @click="preview(record.source)">
                                <i class="bi-play-fill" />
                            </button>
                        </div>
                        <div class="pl-2 pr-3 hidden 2md:block text-right tabular-nums whitespace-nowrap">{{ secToHMS(record.duration) }}</div>
                        <div class="px-2 hidden 2md:block text-right tabular-nums whitespace-nowrap">
                            {{ secToHMS(record.in) }}
                        </div>
                        <div class="px-2 hidden 2md:block text-right tabular-nums whitespace-nowrap">
                            {{ secToHMS(record.out) }}
                        </div>
                        <div class="text-center hidden 2md:block leading-3">
                            <input
                                class="checkbox checkbox-xs rounded"
                                type="checkbox"
                                :checked="record.category && record.category === 'advertisement' ? true : false"
                                @change="setCategory($event, record)"
                            />
                        </div>
                        <div class="text-center hover:text-base-content/70">
                            <button class="cursor-pointer" @click="editItem(index)">
                                <i class="bi-pencil-square" />
                            </button>
                        </div>
                        <div
                            class="pl-2 pr-3 hidden 2md:flex justify-center text-base-content/80 hover:text-base-content/60"
                        >
                            <button class="cursor-pointer" @click="deletePlaylistItem(index)">
                                <i class="bi-x-circle-fill" />
                            </button>
                        </div>
                    </li>
                </template>
            </VirtualList>
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

import VirtualList from 'vue-virtual-draglist'

import { ref, computed, nextTick, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useWindowSize, until } from '@vueuse/core'
import { storeToRefs } from 'pinia'

dayjs.extend(customParseFormat)
dayjs.extend(LocalizedFormat)
dayjs.extend(timezone)
dayjs.extend(utc)

import { stringFormatter, playlistOperations, isLivePlaylistItem } from '@/composables/helper'
import { useConfig } from '@/stores/config'
import { useMedia } from '@/stores/media'
import { usePlaylist } from '@/stores/playlist'

const { t } = useI18n()
const { width } = useWindowSize({ initialWidth: 800 })
const configStore = useConfig()
const mediaStore = useMedia()
const playlistStore = usePlaylist()
const { secToHMS, filename, secondsToTime } = stringFormatter()
const { processPlaylist, genUID } = playlistOperations()

// label scheduled live blocks (category/type/source "live") as LIVE
const isLiveItem = isLivePlaylistItem

const playlistContainer = ref()
const sortContainer = ref()
const todayDate = computed(() => dayjs().tz(configStore.timezone).format('YYYY-MM-DD'))
const { i } = storeToRefs(useConfig())
const { currentIndex, listDate, playoutIsRunning, scrollToItem, missingSources } =
    storeToRefs(usePlaylist())

const dragGroup = ref({ name: 'dragGroup', pull: true, put: true })

const missingSet = computed(() => new Set(missingSources.value))

const sourcesKey = computed(() =>
    playlistStore.playlist
        .map((item) => item.source)
        .filter(Boolean)
        .sort()
        .join('|'),
)

// column tracks must match in header and rows; without a start column
// (infinit mode) the freed 85px track would squeeze the file column,
// so both variants get their own grid definition
const gridClass = computed(() =>
    configStore.playout.playlist.infinit
        ? 'grid-cols-[auto_85px] 2md:grid-cols-[auto_150px_85px_85px_85px_85px_85px_85px_85px_85px_85px]'
        : 'grid-cols-[85px_auto_85px_85px] 2md:grid-cols-[85px_auto_150px_85px_85px_85px_85px_85px_85px_85px_85px_85px]',
)

// row state colors, highest priority first:
// red - source file missing from storage, green - currently playing clip,
// amber - overtime, blue - cued next event, grey - already played clips
function rowState(record: PlaylistItem, index: number) {
    const live = playlistStore.playoutIsRunning && listDate.value === todayDate.value
    const cls: Record<string, boolean> = {}

    if (missingSet.value.has(record.source)) {
        cls['bg-red-500/25!'] = true
        return cls
    }

    if (live) {
        if (index === currentIndex.value) {
            if (record.overtime) {
                cls['bg-amber-600/40!'] = true
            } else {
                cls['bg-green-500/30!'] = true
            }
        } else if (index === currentIndex.value + 1) {
            cls['bg-blue-400/25!'] = true
        } else if (index < currentIndex.value) {
            cls['bg-neutral-400/20!'] = true
            cls['text-base-content/40'] = true
            return cls
        }
    } else if (record.overtime) {
        cls['bg-amber-600/40!'] = true
    }

    if (record.category === 'advertisement') {
        cls['text-base-content/60'] = true
    }

    return cls
}

function isCurrentItem(index: number) {
    return playlistStore.playoutIsRunning && index === currentIndex.value
}

const currentProgress = computed(() => {
    if (!playlistStore.playoutIsRunning) {
        return 0
    }

    const p = playlistStore.progressValue
    return p > 100 ? 100 : p < 0 ? 0 : p
})

function nameTitle(record: PlaylistItem) {
    const name = (record.asset_id || record.title || filename(record.source)).replace(/\.[^.]+$/, '')
    const seg =
        record.segment && record.segment_count
            ? ` — Part ${record.segment}/${record.segment_count} (${secToHMS(record.in)} - ${secToHMS(record.out)})`
            : ''
    const missing = missingSet.value.has(record.source) ? ' - file missing' : ''
    return `${name}${missing}${seg}`
}

function seTooltip(record: PlaylistItem): string {
    return (record.secondary_events || [])
        .map(
            (se) =>
                `${se.template}: ${secToHMS(se.in ?? se.offset)} - ${secToHMS(
                    se.out ?? (se.offset || 0) + (se.duration || 0)
                )}${se.text ? ` "${se.text}"` : ''}`
        )
        .join('\n')
}

defineProps({
    editItem: {
        type: Function,
        default() {
            return ''
        },
    },
    preview: {
        type: Function,
        default() {
            return ''
        },
    },
})

onMounted(() => {
    setTimeout(() => {
        getPlaylist()
    }, 150)

    // Segmentation (cut_segments) lives on the media library assets; make sure
    // it is loaded so drops into the playlist can create rundown segment rows.
    mediaStore.getLibrary()
})

watch([listDate, i], () => {
    setTimeout(() => {
        getPlaylist()
    }, 800)

    mediaStore.getLibrary()
})

let missingCheckTimer = 0

watch(sourcesKey, () => {
    clearTimeout(missingCheckTimer)
    missingCheckTimer = window.setTimeout(() => {
        playlistStore.checkMissingSources()
    }, 250)
})

watch([playoutIsRunning, scrollToItem], () => {
    if (playoutIsRunning.value || scrollToItem.value) {
        setTimeout(() => {
            scrollTo(currentIndex.value)

            scrollToItem.value = false
        }, 400)
    }
})

defineExpose({
    getPlaylist,
})

function scrollTo(index: number) {
    sortContainer.value.scrollToIndex(index)
}

async function getPlaylist() {
    playlistStore.isLoading = true
    await playlistStore.getPlaylist(listDate.value)
    playlistStore.isLoading = false

    playlistStore.checkMissingSources()

    if (listDate.value === todayDate.value) {
        await until(currentIndex).toMatch((v) => v > 0, { timeout: 1500 })
        scrollTo(currentIndex.value)
    } else {
        scrollTo(0)
    }
}

function setCategory(event: any, item: PlaylistItem) {
    if (event.target.checked) {
        item.category = 'advertisement'
    } else {
        item.category = ''
    }
}

function addBG(item: any) {
    if (item.event?.target) {
        item.event?.target.classList.add('!bg-fuchsia-900/30')
    } else {
        item.classList?.add('!bg-fuchsia-900/30')
    }
}

function removeBG(item: any) {
    setTimeout(() => {
        item.classList?.remove('!bg-fuchsia-900/30')
    }, 1000)
}

async function dropItem(event: any) {
    const nIndex = event.newIndex

    if (event.event.from.id === 'mediaList') {
        const media = event.item
        const storagePath = configStore.channels[configStore.i]?.storage
        const sourcePath = `${storagePath}/${mediaStore.folderTree.source}/${media.name}`.replace(/\/[/]+/g, '/')

        // Segmented assets are broadcast "rundown events": each kept segment is
        // inserted as its own playlist row with SOM = segment start and
        // EOM = segment end against the same master file. The engine seeks into
        // the master file per row, so only the cut part plays - not the whole file.
        const asset = await resolveAssetForDrop(media.name || '')
        const assetStem = String(media.name || '').replace(/\.[^.]+$/, '')
        const cutSegments = asset?.cut_segments?.length ? asset.cut_segments : null

        const rows = cutSegments
            ? cutSegments.map((s, i) => ({
                  uid: genUID(),
                  begin: 0,
                  source: sourcePath,
                  in: s.start,
                  out: s.end,
                  duration: Math.max(0, s.end - s.start),
                  segment: i + 1,
                  segment_count: cutSegments.length,
                  asset_id: asset?.asset_id ?? assetStem,
              }))
            : (() => {
                  // Unsegmented: honour TC IN / TC OUT (fall back to 0 -> full).
                  const mediaTcIn = asset && asset.tc_in != null ? asset.tc_in : 0
                  const mediaTcOut =
                      asset && (asset.tc_out != null || asset.tc_in != null)
                          ? asset.tc_out ?? mediaTcIn + (media.duration || 10)
                          : media.duration || 10
                  return [
                      {
                          uid: genUID(),
                          begin: 0,
                          source: sourcePath,
                          in: mediaTcIn,
                          out: mediaTcOut,
                          duration: media.duration || 10,
                          asset_id: asset?.asset_id ?? assetStem,
                      },
                  ]
              })()

        playlistStore.playlist.splice(nIndex, 0, ...rows)

        nextTick(() => {
            rows.forEach((r) => {
                const newNode = document.getElementById(r.uid)
                addBG(newNode)
                removeBG(newNode)
            })
        })
    }

    processPlaylist(listDate.value, playlistStore.playlist, false)

    nextTick(() => {
        removeBG(event.event.node)

        if (nIndex > playlistStore.playlist.length - 4) {
            sortContainer.value.scrollToBottom()
        }
    })
}

// Resolve the library asset behind a dropped file name so its custom segment
// cuts (defined in the media library -> asset details -> Segments tab) can be
// expanded into rundown rows. Falls back to filtering by asset_id, title base
// or filename base, and reloads the library once if it is not yet loaded.
async function resolveAssetForDrop(name: string): Promise<MediaAsset | null> {
    if (!mediaStore.library.length) {
        await mediaStore.getLibrary()
    }

    const stem = String(name || '').replace(/\.[^.]+$/, '')
    const base = stem.toLowerCase()
    const candidates = mediaStore.library.filter(
        (a) => a.category === 'videos' || a.category === 'rescue'
    )

    return (
        candidates.find((a) => a.asset_id.toLowerCase() === base) ||
        candidates.find((a) => (a.title || '').replace(/\.[^.]+$/, '').toLowerCase() === base) ||
        candidates.find((a) => {
            const file = String(a.path || '').split('/').pop() || ''
            return file.replace(/\.[^.]+$/, '').toLowerCase() === base
        }) ||
        null
    )
}

function deletePlaylistItem(index: number) {
    playlistStore.playlist.splice(index, 1)

    processPlaylist(listDate.value, playlistStore.playlist, false)
}
</script>
<style>
#sort-container .timeHidden {
    display: none !important;
}

/*
    format dragging elements
*/
.media-placeholder,
.sortable-ghost {
    background-color: #701a754b !important;
    min-height: 37px !important;
    height: 37px !important;
}

#playlist-container .media-placeholder div:nth-child(1)::after {
    content: '00:00:00';
    padding-left: 4px;
}

#playlist-container .media-placeholder div:nth-child(1) i {
    display: none;
}

#playlist-container .media-placeholder div:nth-child(4),
#playlist-container .media-placeholder div:nth-child(5),
#playlist-container .media-placeholder div:nth-child(6) {
    text-align: right;
    padding-right: 12px;
}

#playlist-container .media-placeholder {
    grid-template-columns: 85px auto 85px;
}

@media (max-width: 875px) {
    #playlist-container .media-placeholder div:nth-child(4) {
        display: none;
    }
}

@media (min-width: 876px) {
    #playlist-container .media-placeholder {
        grid-template-columns: 85px auto 150px 85px 85px 85px 85px 85px 85px 85px 85px 85px;
    }

    #playlist-container .media-placeholder div {
        display: block;
    }
}
</style>
