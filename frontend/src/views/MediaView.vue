<template>
    <div class="h-[calc(100vh-140px)] px-2 overflow-hidden">
        <section class="h-full border border-base-content/30 rounded-sm bg-base-100 overflow-hidden">
            <div class="flex h-full">
                <aside class="w-52 shrink-0 border-r border-base-content/20 bg-base-200 overflow-y-auto flex flex-col">
                    <div class="px-4 py-3 font-bold text-base-content/80">Media Library Assets</div>
                    <ul class="menu menu-sm p-2">
                        <li
                            v-for="cat in libraryCategories"
                            :key="cat.key"
                            @click="setCategory(cat.key)"
                        >
                            <a :class="{ active: libraryCategory === cat.key }">
                                <span class="flex-1">{{ cat.label }}</span>
                                <span class="badge badge-sm badge-ghost">{{ libraryCount(cat.key) }}</span>
                            </a>
                        </li>
                    </ul>

                    <div class="mt-auto px-4 py-3 border-t border-base-content/20">
                        <div class="text-xs font-bold text-base-content/80 mb-1">STORAGE OVERVIEW</div>
                        <div class="text-sm text-base-content/90">
                            {{ storageUsedText }} used | {{ storageFilesText }}
                        </div>
                    </div>
                </aside>

                <div class="flex-1 min-w-0 flex flex-col">
                    <div class="flex items-center justify-between gap-2 px-4 py-2 border-b border-base-content/20">
                        <h2 class="text-lg font-bold">{{ currentLibraryLabel }}</h2>
                        <div class="flex items-center gap-2">
                            <button
                                v-if="libraryCategory === 'logo'"
                                class="btn btn-xs btn-primary"
                                :disabled="logoProcessing"
                                @click="processAnimatedLogo"
                            >
                                <span v-if="logoProcessing" class="loading loading-spinner loading-xs" />
                                {{ t('media.processLogo') }}
                            </button>
                            <input
                                v-model="librarySearch"
                                type="text"
                                class="input input-sm w-72"
                                placeholder="Search by Title or Asset ID"
                            />
                        </div>
                    </div>

                    <div class="flex items-center gap-2 px-4 py-2 text-sm text-base-content/70 border-b border-base-content/20">
                        <button
                            v-for="tab in statusTabs"
                            :key="tab.key"
                            class="btn btn-xs"
                            :class="libraryStatus === tab.key ? 'btn-primary' : 'btn-ghost'"
                            @click="libraryStatus = tab.key"
                        >
                            {{ tab.label }}: {{ libraryStatusCount(tab.key) }}
                        </button>
                    </div>

                    <div
                        v-if="selectedAssets.length"
                        class="toolbar flex items-center gap-2 px-4 py-2 border-b border-base-content/20"
                    >
                        <span class="text-sm text-base-content/70">
                            {{ selectedAssets.length }} selected
                        </span>
                        <button class="btn btn-xs btn-primary" @click="reprocessSelected">
                            ↻ Reprocess
                        </button>
                        <button class="btn btn-xs" @click="archiveSelected">🗄 Archive</button>
                        <button class="btn btn-xs btn-error" @click="deleteSelected">🗑 Delete</button>
                        <button class="btn btn-xs btn-ghost ml-auto" @click="clearSelection">
                            ✕ Clear
                        </button>
                    </div>

                    <div v-if="mediaStore.libraryLoading" class="flex items-center justify-center py-8">
                        <span class="loading loading-spinner loading-lg" />
                    </div>

                    <div v-else class="relative flex-1 overflow-auto">
                        <table class="table table-zebra table-sm w-full">
                            <thead>
                                <tr>
                                    <th></th>
                                    <th>Asset Information</th>
                                    <th>Type</th>
                                    <th>Segments</th>
                                    <th>Duration</th>
                                    <th>Audio</th>
                                    <th>CC</th>
                                    <th>Status</th>
                                    <th>Frame Rate</th>
                                    <th>Preview</th>
                                </tr>
                            </thead>
                            <tbody>
                                <tr v-if="filteredLibrary.length === 0">
                                    <td colspan="10" class="text-center text-base-content/50 py-8">
                                        No assets found.
                                    </td>
                                </tr>
                                <tr
                                    v-for="asset in filteredLibrary"
                                    :key="asset.path"
                                    :class="{ 'bg-primary/10': isSelected(asset) }"
                                    class="cursor-pointer"
                                    @click="toggleSelect(asset)"
                                >
                                    <td class="w-8">
                                        <input
                                            type="checkbox"
                                            class="checkbox checkbox-xs"
                                            :checked="isSelected(asset)"
                                            @click.stop="toggleSelect(asset)"
                                        />
                                    </td>
                                    <td>
                                        <button class="font-bold" @click.stop="openAsset(asset)">
                                            {{ displayTitle(asset) }}
                                        </button>
                                    </td>
                                    <td>
                                        {{ typeLabel(asset) }}
                                    </td>
                                    <td>{{ asset.segments }}</td>
                                    <td>{{ asset.duration }}</td>
                                    <td>
                                        <span
                                            v-if="asset.audio > 0"
                                            class="badge badge-sm border border-success text-success"
                                        >
                                            {{ asset.audio }}
                                        </span>
                                        <span v-else class="text-base-content/40">0</span>
                                    </td>
                                    <td>
                                        <span
                                            class="badge badge-sm"
                                            :class="asset.cc ? 'border border-success text-success' : 'border border-base-content/40 text-base-content/40'"
                                        >
                                            CC {{ asset.cc ? 1 : 0 }}
                                        </span>
                                    </td>
                                    <td>
                                        <span class="badge badge-sm" :class="statusClass(asset.status)">
                                            {{ asset.status }}
                                        </span>
                                    </td>
                                    <td>{{ asset.frame_rate }}</td>
                                    <td>
                                        <button
                                            v-if="(asset.category === 'graphics' || asset.category === 'logo') && asset.graphic_source"
                                            class="btn btn-xs btn-primary me-1"
                                            :disabled="asset.category === 'logo' ? isProcessingLogo(asset.path) : isProcessingGraphic(asset.path)"
                                            :title="t('media.processGraphic')"
                                            @click.stop="asset.category === 'logo' ? processLogo(asset) : processGraphic(asset)"
                                        >
                                            <span
                                                v-if="asset.category === 'logo' ? isProcessingLogo(asset.path) : isProcessingGraphic(asset.path)"
                                                class="loading loading-spinner loading-xs"
                                            />
                                            <i v-else class="bi-magic" />
                                            {{ t('media.processGraphic') }}
                                        </button>
                                        <button
                                            class="btn btn-xs btn-ghost mt-1"
                                            :title="t('media.preview')"
                                            @click.stop="showPreview(asset)"
                                        >
                                            <i class="bi-eye" />
                                            {{ t('media.preview') }}
                                        </button>
                                    </td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </section>

        <!-- Preview Modal -->
        <dialog ref="previewModalRef" class="modal" @close="onPreviewClose">
            <div class="modal-box max-w-5xl p-0 bg-base-100 rounded-lg overflow-hidden">
                <div class="flex items-center justify-between gap-2 px-4 py-3 bg-base-300/40 border-b border-base-content/20">
                    <div class="font-bold text-base truncate">
                        Asset Details: <span class="text-primary">{{ previewAsset?.asset_id }}</span>
                    </div>
                    <button class="btn btn-sm btn-ghost btn-circle shrink-0" @click="closePreview">
                        <i class="bi-x-lg text-lg" />
                    </button>
                </div>

                <div class="flex flex-col lg:flex-row gap-4 p-4">
                    <div class="lg:w-[58%] shrink-0">
                        <div class="aspect-video bg-black rounded-box flex items-center justify-center overflow-hidden relative">
                            <video
                                v-if="previewKind === 'video' && previewAsset"
                                ref="previewVideoRef"
                                :key="previewAsset.play_path"
                                class="w-full h-full"
                                controls
                                autoplay
                                crossorigin="anonymous"
                                :loop="previewAsset.category === 'logo'"
                                @timeupdate="onSegmentTimeupdate"
                                @play="segmentPlaying = true"
                                @pause="segmentPlaying = false"
                            >
                                <source :src="previewSrc" :type="previewMime" />
                                <track
                                    v-if="previewSubtitleUrl"
                                    kind="subtitles"
                                    label="CC 1"
                                    srclang="en"
                                    :src="previewSubtitleUrl"
                                    default
                                />
                                Your browser does not support the video tag.
                            </video>
                            <AudioMeter v-if="previewKind === 'video' && previewAsset" :media="previewVideoRef" />
                            <div
                                v-else-if="previewKind === 'audio' && previewAsset"
                                class="text-center"
                            >
                                <i class="bi-music-note-beamed text-5xl text-base-content/50" />
                                <audio :src="previewSrc" controls class="mt-4 w-72" />
                            </div>
                            <img
                                v-else-if="previewKind === 'image' && previewAsset"
                                :src="previewSrc"
                                :alt="previewAsset.title"
                                class="max-h-full max-w-full object-contain"
                            />
                            <div v-else class="flex flex-col items-center justify-center gap-4 text-center px-6">
                                <p class="text-base-content/60">Preview Not Available</p>
                                <div class="w-[220px] h-[160px] flex items-center justify-center border border-dashed border-base-content/30 rounded-box bg-base-200">
                                    <i class="bi-file-earmark-media text-4xl text-base-content/40" />
                                </div>
                            </div>
                        </div>
                        <div class="mt-3 flex flex-wrap items-center gap-2 text-sm">
                            <span class="badge badge-sm" :class="statusClass(previewAsset?.status || '')">
                                {{ previewAsset?.status }}
                            </span>
                            <span class="break-all text-base-content/60"><!--{{ previewFullPath }} --></span>
                        </div>
                    </div>

                    <div class="flex-1 min-w-0">
                        <div class="border border-base-content/20 rounded-box overflow-hidden">
                            <div class="flex items-center justify-between px-4 py-2 bg-base-300/40 border-b border-base-content/20 font-bold text-sm">
                                <span>Asset Information</span>
                                <span v-if="previewAsset?.cc" class="badge badge-sm badge-outline">CC 1</span>
                            </div>
                            <div class="text-sm">
                                <div class="flex items-center justify-between gap-4 px-4 py-2 border-b border-base-content/10">
                                    <span class="text-base-content/60 shrink-0">Title</span>
                                    <span v-if="editingField !== 'title'" class="flex items-center gap-2 min-w-0 text-right">
                                        <span class="truncate">{{ previewTitle }}</span>
                                        <button class="btn btn-xs btn-ghost" title="Edit Title" @click="openFieldEdit('title')">
                                            <i class="bi-pencil" />
                                        </button>
                                    </span>
                                    <span v-else class="flex items-center gap-1 min-w-0">
                                        <input
                                            v-model="editValue"
                                            class="input input-bordered input-xs w-44 text-right font-mono"
                                            @keydown.enter="saveFieldEdit"
                                            @keydown.esc="cancelFieldEdit"
                                            autofocus
                                        />
                                        <button class="btn btn-xs btn-ghost text-success" @click="saveFieldEdit">
                                            <i class="bi-check-lg" />
                                        </button>
                                        <button class="btn btn-xs btn-ghost" @click="cancelFieldEdit">
                                            <i class="bi-x-lg" />
                                        </button>
                                    </span>
                                </div>
                                <div class="flex items-center justify-between gap-4 px-4 py-2 border-b border-base-content/10">
                                    <span class="text-base-content/60 shrink-0">Asset ID</span>
                                    <span v-if="editingField !== 'asset_id'" class="flex items-center gap-2 min-w-0 text-right">
                                        <span class="truncate">{{ previewAsset?.asset_id }}</span>
                                        <button class="btn btn-xs btn-ghost" title="Edit Asset ID" @click="openFieldEdit('asset_id')">
                                            <i class="bi-pencil" />
                                        </button>
                                    </span>
                                    <span v-else class="flex items-center gap-1 min-w-0">
                                        <input
                                            v-model="editValue"
                                            class="input input-bordered input-xs w-40 text-right font-mono"
                                            @keydown.enter="saveFieldEdit"
                                            @keydown.esc="cancelFieldEdit"
                                            autofocus
                                        />
                                        <button class="btn btn-xs btn-ghost text-success" @click="saveFieldEdit">
                                            <i class="bi-check-lg" />
                                        </button>
                                        <button class="btn btn-xs btn-ghost" @click="cancelFieldEdit">
                                            <i class="bi-x-lg" />
                                        </button>
                                    </span>
                                </div>
                                <div class="flex justify-between gap-4 px-4 py-2 border-b border-base-content/10">
                                    <span class="text-base-content/60">Category</span>
                                    <span>{{ previewAsset?.category }}</span>
                                </div>
                                <div class="flex justify-between gap-4 px-4 py-2 border-b border-base-content/10">
                                    <span class="text-base-content/60">Type</span>
                                    <span>{{ previewAsset?.media_type }}</span>
                                </div>
                                <div class="flex justify-between gap-4 px-4 py-2 border-b border-base-content/10">
                                    <span class="text-base-content/60">Priority</span>
                                    <span>0</span>
                                </div>
                                <div class="flex justify-between gap-4 px-4 py-2 border-b border-base-content/10">
                                    <span class="text-base-content/60">Duration</span>
                                    <span>{{ previewAsset?.duration }}</span>
                                </div>
                                <div class="flex justify-between gap-4 px-4 py-2 border-b border-base-content/10">
                                    <span class="text-base-content/60">Status</span>
                                    <span>
                                        <span class="badge badge-sm" :class="statusClass(previewAsset?.status || '')">
                                            {{ previewAsset?.status }}
                                        </span>
                                    </span>
                                </div>
                                <div class="flex justify-between gap-4 px-4 py-2 border-b border-base-content/10">
                                    <span class="text-base-content/60">File Size</span>
                                    <span>{{ fileSize }}</span>
                                </div>
                                <div class="flex items-center justify-between gap-4 px-4 py-2 border-b border-base-content/10">
                                    <span class="text-base-content/60 shrink-0">TC IN</span>
                                    <span v-if="editingField !== 'tc'" class="flex items-center gap-2 min-w-0 text-right">
                                        <span class="font-mono">{{ tcIn }}</span>
                                        <button class="btn btn-xs btn-ghost" title="Edit TC IN (SOM in playlist)" @click="openFieldEdit('tc')">
                                            <i class="bi-pencil" />
                                        </button>
                                    </span>
                                    <span v-else class="flex items-center gap-1 min-w-0">
                                        <input
                                            v-model="editValue"
                                            class="input input-bordered input-xs w-32 text-right font-mono"
                                            @keydown.enter="saveFieldEdit"
                                            @keydown.esc="cancelFieldEdit"
                                            autofocus
                                        />
                                        <button class="btn btn-xs btn-ghost text-success" @click="saveFieldEdit">
                                            <i class="bi-check-lg" />
                                        </button>
                                        <button class="btn btn-xs btn-ghost" @click="cancelFieldEdit">
                                            <i class="bi-x-lg" />
                                        </button>
                                    </span>
                                </div>
                                <div class="flex items-center justify-between gap-4 px-4 py-2 border-b border-base-content/10">
                                    <span class="text-base-content/60 shrink-0">TC OUT</span>
                                    <span v-if="editingField !== 'tc_out'" class="flex items-center gap-2 min-w-0 text-right">
                                        <span class="font-mono">{{ tcOut }}</span>
                                        <button class="btn btn-xs btn-ghost" title="Edit TC OUT (EOM in playlist)" @click="openFieldEdit('tc_out')">
                                            <i class="bi-pencil" />
                                        </button>
                                    </span>
                                    <span v-else class="flex items-center gap-1 min-w-0">
                                        <input
                                            v-model="editValue"
                                            class="input input-bordered input-xs w-32 text-right font-mono"
                                            @keydown.enter="saveFieldEdit"
                                            @keydown.esc="cancelFieldEdit"
                                            autofocus
                                        />
                                        <button class="btn btn-xs btn-ghost text-success" @click="saveFieldEdit">
                                            <i class="bi-check-lg" />
                                        </button>
                                        <button class="btn btn-xs btn-ghost" @click="cancelFieldEdit">
                                            <i class="bi-x-lg" />
                                        </button>
                                    </span>
                                </div>
                                <div class="flex justify-between gap-4 px-4 py-2">
                                    <span class="text-base-content/60">Bitrate</span>
                                    <span>{{ bitrate }}</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="flex items-center overflow-x-auto border-t border-base-content/20 px-2">
                    <button
                        v-for="tab in assetTabs"
                        :key="tab.key"
                        class="btn btn-sm btn-ghost rounded-none px-3 border-b-2"
                        :class="activeTab === tab.key ? 'border-primary text-primary' : 'border-transparent text-base-content/50'"
                        @click="activeTab = tab.key"
                    >
                        {{ tab.label }}
                    </button>
                </div>

                <div class="p-4">
                    <div v-if="activeTab === 'streams'" class="grid grid-cols-1 md:grid-cols-3 gap-4">
                        <div class="border border-base-content/20 rounded-box overflow-hidden">
                            <div class="flex items-center justify-between px-3 py-2 bg-base-300/40 border-b border-base-content/20 font-bold text-sm">
                                <span>Video</span>
                                <span>{{ videoStreamInfo.streams }} Stream</span>
                            </div>
                            <div class="max-h-60 overflow-y-auto text-sm">
                                <div v-if="!videoStreamInfo.entries.length" class="px-3 py-2 text-base-content/50">
                                    No video stream
                                </div>
                                <div
                                    v-for="item in videoStreamInfo.entries"
                                    :key="item.label"
                                    class="flex justify-between gap-3 px-3 py-2 border-b border-base-content/10"
                                >
                                    <span class="text-base-content/60">{{ item.label }}</span>
                                    <span class="text-right">{{ item.value }}</span>
                                </div>
                            </div>
                        </div>

                        <div class="border border-base-content/20 rounded-box overflow-hidden">
                            <div class="flex items-center justify-between px-3 py-2 bg-base-300/40 border-b border-base-content/20 font-bold text-sm">
                                <span>Audio</span>
                                <span>{{ audioStreamInfo.length }} {{ audioStreamInfo.length === 1 ? 'Track' : 'Tracks' }}</span>
                            </div>
                            <div class="max-h-60 overflow-y-auto text-sm">
                                <div v-if="!audioStreamInfo.length" class="px-3 py-2 text-base-content/50">
                                    No audio track
                                </div>
                                <div
                                    v-for="(stream, i) in audioStreamInfo"
                                    :key="i"
                                    class="flex justify-between gap-3 px-3 py-2 border-b border-base-content/10"
                                >
                                    <span class="link">{{ streamLabel(stream, i) }}</span>
                                    <span>{{ streamCodecLabel(stream) }}</span>
                                </div>
                            </div>
                        </div>

                        <div class="border border-base-content/20 rounded-box overflow-hidden">
                            <div class="flex items-center justify-between px-3 py-2 bg-base-300/40 border-b border-base-content/20 font-bold text-sm">
                                <span>Subtitles</span>
                                <span>{{ subtitleStreamInfo.length }} {{ subtitleStreamInfo.length === 1 ? 'Language' : 'Languages' }}</span>
                            </div>
                            <div class="max-h-60 overflow-y-auto text-sm">
                                <div v-if="!subtitleStreamInfo.length" class="px-3 py-2 text-base-content/50">
                                    No subtitle stream
                                </div>
                                <div
                                    v-for="(stream, i) in subtitleStreamInfo"
                                    :key="i"
                                    class="flex justify-between gap-3 px-3 py-2 border-b border-base-content/10"
                                >
                                    <span class="link">{{ streamLabel(stream, i) }}</span>
                                    <span>{{ streamCodecLabel(stream) }}</span>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div v-else-if="activeTab === 'segments'">
                        <div v-if="!segmentBlocks.length" class="border border-base-content/20 rounded-box px-4 py-6 text-center text-sm text-base-content/50">
                            No segments available for this asset.
                        </div>
                        <template v-else>
                            <div class="flex flex-wrap items-center justify-center gap-2 mb-4">
                                <button
                                    class="btn btn-sm btn-ghost border border-base-content/20"
                                    title="Previous segment"
                                    :disabled="!isVideoPreview || currentSegment === 0"
                                    @click="seekSegment(-1)"
                                >
                                    <i class="bi-skip-start" />
                                </button>
                                <button
                                    class="btn btn-sm btn-ghost border border-base-content/20"
                                    :title="segmentPlaying ? 'Pause' : 'Play'"
                                    :disabled="!isVideoPreview"
                                    @click="toggleSegmentPlay"
                                >
                                    <i :class="segmentPlaying ? 'bi-pause-fill' : 'bi-play-fill'" />
                                </button>
                                <button
                                    class="btn btn-sm btn-ghost border border-base-content/20"
                                    title="Next segment"
                                    :disabled="!isVideoPreview || currentSegment >= segmentBlocks.length - 1"
                                    @click="seekSegment(1)"
                                >
                                    <i class="bi-skip-end" />
                                </button>
                                <input
                                    v-model="segmentTimecode"
                                    class="input input-sm w-32 text-center font-mono"
                                    :disabled="!isVideoPreview"
                                    @keydown.enter="seekToTimecode"
                                />
                                <button
                                    class="btn btn-sm btn-ghost border border-base-content/20"
                                    title="Go to timecode"
                                    :disabled="!isVideoPreview"
                                    @click="seekToTimecode"
                                >
                                    <i class="bi-search" />
                                </button>
                            </div>

                            <div class="flex items-stretch gap-1 h-10 mb-3">
                                <div
                                    v-for="(seg, i) in segmentBlocks"
                                    :key="i"
                                    class="rounded-sm cursor-pointer flex items-center justify-center text-xs font-mono border select-none"
                                    :class="i === currentSegment ? 'bg-primary text-primary-content border-primary' : 'bg-base-300/40 border-base-content/20 text-base-content/60 hover:bg-base-300/70'"
                                    :style="{ width: seg.width + '%' }"
                                    :title="`Segment ${i + 1}: ${seg.label}`"
                                    @click="seekToSegment(i)"
                                >
                                    {{ segmentBlocks.length <= 12 ? i + 1 : '' }}
                                </div>
                            </div>

                            <div class="flex flex-wrap items-center justify-center gap-x-6 gap-y-1 text-sm text-base-content/60">
                                <span>Segment {{ currentSegment + 1 }} / {{ segmentBlocks.length }}</span>
                                <span>Length: {{ segmentLength }}s</span>
                                <span>Total: {{ previewAsset?.duration }}</span>
                            </div>

                            <div class="mt-4 border border-base-content/20 rounded-box overflow-hidden">
                                <div class="flex items-center justify-between px-3 py-2 bg-base-300/40 border-b border-base-content/20 font-bold text-sm">
                                    <span>Segments Table</span>
                                    <span class="flex items-center gap-2">
                                        <span>{{ segmentTableRows.length }} segments</span>
                                        <button
                                            class="btn btn-xs btn-outline btn-primary"
                                            :disabled="!isVideoPreview || segmentsSaving"
                                            title="Split active segment at the playhead"
                                            @click="createSegment"
                                        >
                                            <i class="bi-scissors" />
                                            Create Segment
                                        </button>
                                    </span>
                                </div>
                                <div class="max-h-72 overflow-y-auto text-sm">
                                    <table class="table table-sm">
                                        <thead>
                                            <tr class="text-base-content/60">
                                                <th class="w-12">#</th>
                                                <th>Start</th>
                                                <th>End</th>
                                                <th>Duration</th>
                                                <th class="w-14 text-center">Play</th>
                                                <th class="w-14 text-center">Edit</th>
                                                <th class="w-14 text-center">Delete</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            <tr
                                                v-for="(seg, i) in segmentTableRows"
                                                :key="seg.index"
                                                :class="{ 'bg-primary/10': i === currentSegment }"
                                            >
                                                <td>{{ i + 1 }}</td>
                                                <td class="font-mono">
                                                    <input
                                                        v-if="editingSegment === i"
                                                        v-model="editSegStart"
                                                        class="input input-bordered input-xs w-28 font-mono"
                                                        @keydown.enter="saveSegmentEdit(i)"
                                                        @keydown.esc="cancelSegmentEdit"
                                                        autofocus
                                                    />
                                                    <span v-else>{{ seg.start }}</span>
                                                </td>
                                                <td class="font-mono">
                                                    <input
                                                        v-if="editingSegment === i"
                                                        v-model="editSegEnd"
                                                        class="input input-bordered input-xs w-28 font-mono"
                                                        @keydown.enter="saveSegmentEdit(i)"
                                                        @keydown.esc="cancelSegmentEdit"
                                                    />
                                                    <span v-else>{{ seg.end }}</span>
                                                </td>
                                                <td class="font-mono">{{ seg.duration }}</td>
                                                <td class="text-center">
                                                    <button
                                                        class="btn btn-xs btn-ghost"
                                                        :disabled="!isVideoPreview"
                                                        :title="`Go to segment ${i + 1}`"
                                                        @click="seekToSegment(i)"
                                                    >
                                                        <i class="bi-play-fill" />
                                                    </button>
                                                </td>
                                                <td class="text-center">
                                                    <template v-if="editingSegment === i">
                                                        <button class="btn btn-xs btn-ghost text-success" title="Save" @click="saveSegmentEdit(i)">
                                                            <i class="bi-check-lg" />
                                                        </button>
                                                        <button class="btn btn-xs btn-ghost" title="Cancel" @click="cancelSegmentEdit">
                                                            <i class="bi-x-lg" />
                                                        </button>
                                                    </template>
                                                    <button
                                                        v-else
                                                        class="btn btn-xs btn-ghost"
                                                        :title="`Edit segment ${i + 1}`"
                                                        @click="openSegmentEdit(i)"
                                                    >
                                                        <i class="bi-pencil" />
                                                    </button>
                                                </td>
                                                <td class="text-center">
                                                    <button
                                                        class="btn btn-xs btn-ghost text-error"
                                                        :title="`Delete segment ${i + 1}`"
                                                        @click="deleteSegment(i)"
                                                    >
                                                        <i class="bi-trash" />
                                                    </button>
                                                </td>
                                            </tr>
                                        </tbody>
                                    </table>
                                </div>
                            </div>
                        </template>
                    </div>

                    <div v-else-if="activeTab === 'ingest'" class="max-w-2xl">
                        <div class="border border-base-content/20 rounded-box overflow-hidden">
                            <div class="px-3 py-2 bg-base-300/40 border-b border-base-content/20 font-bold text-sm">Ingest</div>
                            <div class="text-sm">
                                <div class="flex justify-between gap-3 px-3 py-2 border-b border-base-content/10">
                                    <span class="text-base-content/60">Source</span>
                                    <span class="text-right break-all">{{ previewFullPath }}</span>
                                </div>
                                <div class="flex justify-between gap-3 px-3 py-2 border-b border-base-content/10">
                                    <span class="text-base-content/60">Asset ID</span>
                                    <span>{{ previewAsset?.asset_id }}</span>
                                </div>
                                <div class="flex justify-between gap-3 px-3 py-2">
                                    <span class="text-base-content/60">Elements</span>
                                    <span>1</span>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div v-else-if="activeTab === 'availability'" class="max-w-2xl">
                        <div class="border border-base-content/20 rounded-box overflow-hidden">
                            <div class="flex items-center justify-between px-3 py-2 bg-base-300/40 border-b border-base-content/20 font-bold text-sm">
                                <span>Availability</span>
                                <span v-if="availabilityLoading" class="loading loading-spinner loading-xs" />
                            </div>
                            <div class="px-3 py-3 text-sm space-y-3">
                                <div class="flex items-center justify-between gap-2 flex-wrap">
                                    <div class="text-base-content/80">
                                         "{{ previewAsset?.asset_id }}"
                                    </div>
                                    <label v-if="availabilityStates.some((s) => s.own_source)" class="flex items-center gap-2 text-xs text-base-content/60" title="Edit which channel keeps the source file">
                                        Source
                                        <select
                                            v-model="availabilityOwner"
                                            class="select select-bordered select-xs font-normal"
                                            :disabled="linking || availabilityLoading"
                                            @change="onAvailabilityOwnerChange"
                                        >
                                            <option v-for="s in availabilityStates.filter((x) => x.own_source)" :key="s.channel_id" :value="s.channel_id">
                                                {{ s.channel_name }}
                                            </option>
                                        </select>
                                        <i class="bi-pencil" />
                                    </label>
                                </div>

                                <p v-if="availabilityError" class="text-error text-xs">{{ availabilityError }}</p>

                                <div v-else class="space-y-2">
                                    <div
                                        v-for="s in availabilityStates"
                                        :key="s.channel_id"
                                        class="flex items-center justify-between gap-2 border border-base-content/15 rounded-box px-3 py-2"
                                        :class="{ 'opacity-60': !s.available && !s.own_source }"
                                    >
                                        <label class="flex items-center gap-3 min-w-0" :class="s.own_source ? 'cursor-default' : 'cursor-pointer'">
                                            <input
                                                type="checkbox"
                                                class="checkbox checkbox-sm checkbox-primary"
                                                :checked="s.available"
                                                :disabled="s.own_source || linking"
                                                @change="toggleAvailability(s)"
                                            />
                                            <div class="min-w-0">
                                                <div class="flex items-center gap-2">
                                                    <span class="font-bold">{{ s.channel_name }}</span>
                                                    <span class="badge badge-xs" :class="s.own_source ? 'badge-primary' : s.available ? 'badge-success' : 'badge-ghost'">
                                                        {{ s.own_source ? 'Source' : s.available ? 'Shared' : 'Not available' }}
                                                    </span>
                                                </div>
                                                <div class="text-xs text-base-content/60 truncate">
                                                  <!----  {{ s.ts_path ? s.ts_path + ' · ' + availabilitySize(s.ts_size) : 'No playable output shared to this channel' }} -->
                                                </div>
                                            </div>
                                        </label>
                                        <span v-if="s.own_source" class="text-xs text-base-content/50 shrink-0">keep &amp; share</span>
                                    </div>
                                </div>

                                <div class="text-xs text-base-content/50">
                                    
                                </div>
                            </div>
                        </div>
                    </div>

                    <div v-else-if="activeTab === 'actions'" class="max-w-2xl">
                        <div class="border border-base-content/20 rounded-box overflow-hidden">
                            <div class="flex items-center justify-between px-3 py-2 bg-base-300/40 border-b border-base-content/20 font-bold text-sm">
                                <span>Actions</span>
                                <span class="badge badge-sm" :class="statusClass(previewAsset?.status || '')">{{ previewAsset?.status }}</span>
                            </div>
                            <div class="px-3 py-3 flex flex-wrap gap-2">
                                <button class="btn btn-sm" @click="openEdit">
                                    <i class="bi-pencil" /> Edit
                                </button>
                                <a
                                    v-if="previewAsset?.status !== 'processed'"
                                    class="btn btn-sm btn-primary"
                                    :disabled="isReprocessing"
                                    @click="reprocessPreview"
                                >
                                    <span v-if="isReprocessing" class="loading loading-spinner loading-xs" />
                                    <i v-else class="bi-arrow-repeat" />
                                    {{ isReprocessing ? t('media.processing') : t('media.reprocess') }}
                                </a>
                                <a
                                    class="btn btn-sm"
                                    :href="previewExportUrl"
                                    :download="previewAsset?.title"
                                    :title="t('media.export')"
                                >
                                    <i class="bi-download" /> {{ t('media.export') }}
                                </a>
                                <button class="btn btn-error btn-sm" @click="deletePreview">
                                    <i class="bi-trash" /> {{ t('media.delete') }}
                                </button>
                            </div>
                        </div>
                    </div>

                    <div v-else-if="activeTab === 'qc'" class="max-w-2xl">
                        <div class="border border-base-content/20 rounded-box overflow-hidden">
                            <div class="px-3 py-2 bg-base-300/40 border-b border-base-content/20 font-bold text-sm">QC</div>
                            <div class="text-sm">
                                <div v-for="item in qcRows" :key="item.label" class="flex justify-between gap-3 px-3 py-2 border-b border-base-content/10">
                                    <span class="text-base-content/60">{{ item.label }}</span>
                                    <span>{{ item.value }}</span>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div v-else-if="activeTab === 'metadata'" class="max-w-2xl">
                        <div class="border border-base-content/20 rounded-box overflow-hidden">
                            <div class="px-3 py-2 bg-base-300/40 border-b border-base-content/20 font-bold text-sm">Metadata</div>
                            <div class="text-sm">
                                <div v-for="item in metaRows" :key="item.label" class="flex justify-between gap-3 px-3 py-2 border-b border-base-content/10">
                                    <span class="text-base-content/60">{{ item.label }}</span>
                                    <span class="text-right break-all">{{ item.value }}</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            <form method="dialog" class="modal-backdrop">
                <button>close</button>
            </form>
        </dialog>
        <GenericModal :show="showEdit" :title="t('media.rename')" :modal-action="handleEdit">
            <div class="form-control w-full">
                <label class="label">
                    <span class="label-text">New filename</span>
                </label>
                <input v-model="editName" type="text" class="input input-bordered input-sm w-full" @keydown.enter="handleEdit(true)" />
            </div>
        </GenericModal>
    </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, watch, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { storeToRefs } from 'pinia'
import { useRouter, useRoute } from 'vue-router'
import { useHead } from '@unhead/vue'

import { useConfig } from '@/stores/config'
import { useMedia } from '@/stores/media'
import { useAuth } from '@/stores/auth'
import { useIndex } from '@/stores/index'

import AudioMeter from '@/components/AudioMeter.vue'
import GenericModal from '@/components/GenericModal.vue'

import { stringFormatter } from '@/composables/helper'

const { t } = useI18n()
const router = useRouter()
const route = useRoute()
const configStore = useConfig()
const mediaStore = useMedia()
const authStore = useAuth()
const indexStore = useIndex()
const { i } = storeToRefs(useConfig())

const libraryCategory = ref('videos')
const libraryStatus = ref('all')
const librarySearch = ref('')
const selectedPaths = ref<string[]>([])
const logoProcessing = ref(false)
const processingGraphics = ref<string[]>([])
const processingLogos = ref<string[]>([])

const previewModalRef = ref<HTMLDialogElement | null>(null)
const previewVideoRef = ref<HTMLVideoElement | null>(null)
const previewAsset = ref<MediaAsset | null>(null)
const isReprocessing = ref(false)
const activeTab = ref('streams')
const segmentTimecode = ref('00:00:00:00')
const currentSegment = ref(0)
const segmentPlaying = ref(false)

type EditableField = 'title' | 'asset_id' | 'tc' | 'tc_out'
const editingField = ref<EditableField | null>(null)
const editValue = ref('')
const tcInOverride = ref('')
const tcOutOverride = ref('')
const showEdit = ref(false)
const editName = ref('')
const segmentsCustom = ref<{ start: number; end: number }[] | null>(null)
const segmentsLoading = ref(false)
const segmentsSaving = ref(false)
const editingSegment = ref(-1)
const editSegStart = ref('')
const editSegEnd = ref('')

interface ProbeEntry {
    label: string
    value: string
}

const assetTabs = [
    { key: 'streams', label: 'Video/Audio/Subtitles' },
    { key: 'segments', label: 'Segments' },
    { key: 'ingest', label: 'Ingest' },
    { key: 'availability', label: 'Availability' },
    { key: 'actions', label: 'Actions' },
    { key: 'qc', label: 'QC' },
    { key: 'metadata', label: 'Metadata' },
]

const libraryCategories = [
    { key: 'videos', label: 'Videos' },
    { key: 'graphics', label: 'Graphics' },
    { key: 'logo', label: 'Logos' },
    { key: 'audio', label: 'Audio' },
    { key: 'rescue', label: 'Rescue' },
    { key: 'subtitles', label: 'Subtitles' },
]

const statusTabs = [
    { key: 'all', label: 'All' },
    { key: 'processed', label: 'Processed' },
    { key: 'processing', label: 'Processing' },
    { key: 'failed', label: 'Failed' },
]

const filteredLibrary = computed(() => {
    const query = librarySearch.value.trim().toLowerCase()

    return mediaStore.library.filter((asset) => {
        if (asset.category !== libraryCategory.value) {
            return false
        }

        if (libraryStatus.value !== 'all' && asset.status !== libraryStatus.value) {
            return false
        }

        if (!query) {
            return true
        }

        return (
            asset.title.toLowerCase().includes(query) ||
            asset.asset_id.toLowerCase().includes(query)
        )
    })
})

const selectedAssets = computed(() =>
    mediaStore.library.filter((a) => selectedPaths.value.includes(a.path))
)

const currentLibraryLabel = computed(
    () => libraryCategories.find((c) => c.key === libraryCategory.value)?.label ?? ''
)

document.title = `${t('button.media')} | ffplayout`

onMounted(() => {
    const preset = (route.query.category as string) || 'videos'
    if (libraryCategories.some((c) => c.key === preset)) {
        libraryCategory.value = preset
    }

    mediaStore.getLibrary()
    mediaStore.getStorage()
})

useHead({
    title: computed(() => t('button.media')),
})

watch([i], () => {
    mediaStore.getLibrary()
    mediaStore.getStorage()
})

watch(activeTab, (tab) => {
    if (tab === 'availability') {
        loadAvailability()
    }
})

function setCategory(key: string) {
    libraryCategory.value = key
    clearSelection()
}

function clearSelection() {
    selectedPaths.value = []
}

function isSelected(asset: MediaAsset): boolean {
    return selectedPaths.value.includes(asset.path)
}

function toggleSelect(asset: MediaAsset) {
    if (isSelected(asset)) {
        selectedPaths.value = selectedPaths.value.filter((p) => p !== asset.path)
    } else {
        selectedPaths.value.push(asset.path)
    }
}

function openAsset(asset: MediaAsset) {
    router.push({
        path: `/media/assets/${encodeURIComponent(asset.path)}`,
    })
}

async function reprocessSelected() {
    const sources = selectedAssets.value.map((a) => a.path)
    if (!sources.length) {
        return
    }
    clearSelection()
    await mediaStore.reprocessLibrary(sources)
}

async function processAnimatedLogo() {
    const id = configStore.channels[configStore.i]?.id
    if (!id) {
        return
    }
    logoProcessing.value = true
    try {
        const res = await fetch(`/api/media/${id}/logo/process/`, {
            method: 'POST',
            headers: { ...configStore.contentType, ...authStore.authHeader },
        })
        if (res.status === 404) {
            indexStore.msgAlert('warning', t('media.noAnimatedLogo'), 3)
        } else if (!res.ok) {
            const msg = (await res.text()) || t('media.reprocessError')
            indexStore.msgAlert('error', msg, 5)
        } else {
            const data = await res.json().catch(() => null)
            if (data && data.status === 'failed') {
                indexStore.msgAlert(
                    'error',
                    data.error || t('media.reprocessError'),
                    8,
                )
            } else {
                indexStore.msgAlert(
                    'success',
                    t('media.logoProcessed'),
                    3,
                )
            }
        }
    } catch {
        indexStore.msgAlert('error', t('media.reprocessError'), 3)
    } finally {
        logoProcessing.value = false
        await mediaStore.getLibrary()
    }
}

function isProcessingGraphic(path: string): boolean {
    return processingGraphics.value.includes(path)
}

function isProcessingLogo(path: string): boolean {
    return processingLogos.value.includes(path)
}

function displayTitle(asset: MediaAsset | null | undefined): string {
    if (!asset) return ''
    const t = asset.title || ''
    const i = t.lastIndexOf('.')
    return i > 0 ? t.slice(0, i) : t
}

async function processGraphic(asset: MediaAsset) {
    const id = configStore.channels[configStore.i]?.id
    const source = (asset.graphic_source || asset.process_path || asset.path).replace(/^\//, '')
    if (!id || !source) {
        indexStore.msgAlert('warning', t('media.noGraphic'), 3)
        return
    }
    if (isProcessingGraphic(asset.path)) {
        return
    }
    processingGraphics.value = [...processingGraphics.value, asset.path]
    try {
        const res = await fetch(`/api/media/${id}/graphic/process/`, {
            method: 'POST',
            headers: { ...configStore.contentType, ...authStore.authHeader },
            body: JSON.stringify({ source }),
        })
        if (res.status === 404) {
            indexStore.msgAlert('warning', t('media.noGraphic'), 3)
        } else if (!res.ok) {
            const msg = (await res.text()) || t('media.reprocessError')
            indexStore.msgAlert('error', msg, 5)
        } else {
            const data = await res.json().catch(() => null)
            if (data && data.status === 'failed') {
                indexStore.msgAlert('error', data.error || t('media.reprocessError'), 8)
            } else {
                indexStore.msgAlert('success', t('media.graphicProcessed'), 3)
            }
        }
    } catch {
        indexStore.msgAlert('error', t('media.reprocessError'), 3)
    } finally {
        processingGraphics.value = processingGraphics.value.filter((p) => p !== asset.path)
        await mediaStore.getLibrary()
    }
}

async function processLogo(asset: MediaAsset) {
    const id = configStore.channels[configStore.i]?.id
    const source = (asset.graphic_source || asset.process_path || asset.path).replace(/^\//, '')
    if (!id || !source) {
        indexStore.msgAlert('warning', t('media.noAnimatedLogo'), 3)
        return
    }
    if (isProcessingLogo(asset.path)) {
        return
    }
    processingLogos.value = [...processingLogos.value, asset.path]
    try {
        const res = await fetch(`/api/media/${id}/logo/process/`, {
            method: 'POST',
            headers: { ...configStore.contentType, ...authStore.authHeader },
            body: JSON.stringify({ source }),
        })
        if (res.status === 404) {
            indexStore.msgAlert('warning', t('media.noAnimatedLogo'), 3)
        } else if (!res.ok) {
            const msg = (await res.text()) || t('media.reprocessError')
            indexStore.msgAlert('error', msg, 5)
        } else {
            const data = await res.json().catch(() => null)
            if (data && data.status === 'failed') {
                indexStore.msgAlert('error', data.error || t('media.reprocessError'), 8)
            } else {
                indexStore.msgAlert('success', t('media.logoProcessed'), 3)
            }
        }
    } catch {
        indexStore.msgAlert('error', t('media.reprocessError'), 3)
    } finally {
        processingLogos.value = processingLogos.value.filter((p) => p !== asset.path)
        await mediaStore.getLibrary()
    }
}

async function deleteSelected() {
    const id = configStore.channels[configStore.i]?.id
    const sources = selectedAssets.value.map((a) => a.path).map((p) => p.replace(/^\//, ''))
    if (!sources.length) {
        return
    }
    clearSelection()
    const res = await fetch(`/api/file/${id}/remove/`, {
        method: 'POST',
        headers: { ...configStore.contentType, ...authStore.authHeader },
        body: JSON.stringify(sources),
    })
    if (res.status === 200) {
        indexStore.msgAlert('success', t('media.deleteSuccess'), 3)
    } else {
        indexStore.msgAlert('error', t('media.reprocessError'), 3)
    }
    await mediaStore.getLibrary()
}

function archiveSelected() {
    indexStore.msgAlert('warning', 'Archive not implemented', 3)
}

function libraryCount(category: string): number {
    if (category === 'all') {
        return mediaStore.library.length
    }
    return mediaStore.library.filter((a) => a.category === category).length
}

const storageUsedText = computed(() => {
    const s = mediaStore.storageSummary as Record<string, number>
    const bytes = s.usedBytes ?? s.used_bytes ?? 0
    return stringFormatter().fileSize(bytes, 2)
})

const storageFilesText = computed(() =>
    `${mediaStore.library.length} files`
)

interface AvailabilityChannel {
    channel_id: number
    channel_name: string
    own_source: boolean
    ts_path: string
    ts_size: number
    available: boolean
}

const availabilityLoading = ref(false)
const linking = ref(false)
const availabilityStates = ref<AvailabilityChannel[]>([])
const availabilityOwner = ref<number | null>(null)
const availabilityError = ref('')

function availabilitySize(bytes: number): string {
    const s = stringFormatter().fileSize(bytes, 1)
    return typeof s === 'string' ? s : `${bytes} B`
}

async function loadAvailability() {
    const a = previewAsset.value
    if (!a) return
    availabilityLoading.value = true
    availabilityError.value = ''
    try {
        const res = await fetch(`/api/media/availability/asset/${encodeURIComponent(a.asset_id)}`, {
            headers: { ...authStore.authHeader },
        })
        const data = await res.json()
        if (!res.ok) throw new Error(data.detail || 'Failed to load availability')
        availabilityStates.value = data.channels || []
        availabilityOwner.value = data.owner ?? null
    } catch (e: any) {
        availabilityError.value = e?.message || 'Failed to load availability'
        availabilityStates.value = []
        availabilityOwner.value = null
    } finally {
        availabilityLoading.value = false
    }
}

async function applyAvailability(
    channels: { id: number; available: boolean }[],
    owner: number | null
) {
    const a = previewAsset.value
    if (!a || owner == null) return
    linking.value = true
    try {
        const res = await fetch(`/api/media/availability/asset/${encodeURIComponent(a.asset_id)}/apply/`, {
            method: 'POST',
            headers: { ...configStore.contentType, ...authStore.authHeader },
            body: JSON.stringify({ owner, channels }),
        })
        const data = await res.json()
        if (!res.ok) throw new Error(data.detail || 'Availability update failed')
        availabilityStates.value = data.channels || []
        availabilityOwner.value = data.owner ?? owner
        indexStore.msgAlert('success', `Availability updated for "${a.asset_id}"`, 3)
        mediaStore.getLibrary()
        mediaStore.getStorage()
    } catch (e: any) {
        indexStore.msgAlert('error', e?.message || 'Availability update failed', 3)
        loadAvailability()
    } finally {
        linking.value = false
    }
}

async function toggleAvailability(ch: AvailabilityChannel) {
    await applyAvailability(
        availabilityStates.value.map((s) => ({
            id: s.channel_id,
            available: s.channel_id === ch.channel_id ? !ch.available : s.available,
        })),
        availabilityOwner.value
    )
}

async function onAvailabilityOwnerChange() {
    await applyAvailability(
        availabilityStates.value.map((s) => ({ id: s.channel_id, available: s.available })),
        availabilityOwner.value
    )
}

function libraryStatusCount(status: string): number {
    if (status === 'all') {
        return mediaStore.library.filter((a) => a.category === libraryCategory.value).length
    }
    return mediaStore.library.filter(
        (a) => a.category === libraryCategory.value && a.status === status
    ).length
}

function statusClass(status: string): string {
    if (status === 'processed') {
        return 'border border-success text-success'
    }
    if (status === 'processing') {
        return 'border border-warning text-warning'
    }
    return 'border border-error text-error'
}

function typeLabel(asset: MediaAsset): string {
    const labels: Record<string, string> = {
        video: 'Video',
        audio: 'Audio',
        image: 'Image',
        subtitle: 'Subtitle',
    }
    return labels[asset.media_type] ?? asset.media_type
}

const previewFullPath = computed(() => {
    const a = previewAsset.value
    if (!a) return ''
    const storage = configStore.channels[configStore.i]?.storage || ''
    const normalizedStorage = storage.replace(/\\/g, '/')
    return `${normalizedStorage}${a.path}`.replace(/\/+/g, '/')
})

const previewSrc = computed(() => {
    if (!previewAsset.value) return ''
    const id = configStore.channels[configStore.i]?.id
    // play_path already carries the /preview/ remux route prefix for the
    // transport-stream / rescue assets that browsers can't decode natively.
    // Fall back to wrapping any raw .ts so it still plays in the browser.
    const playPath = previewAsset.value.play_path || previewAsset.value.path
    let clean = playPath.startsWith('/') ? playPath : `/${playPath}`
    if (
        previewAsset.value.title.toLowerCase().endsWith('.ts') &&
        !clean.includes('/preview/')
    ) {
        clean = `/preview${clean}`
    }
    return `/file/${id}${clean}`
})

const previewSubtitleUrl = computed(() => {
    const a = previewAsset.value
    // Subtitle track (CC 1) for the preview: the backend only reports a
    // `.srt`/`.vtt` sidecar when a matching file exists (cc flag).
    if (!a || !a.subtitle) return ''
    const id = configStore.channels[configStore.i]?.id
    const clean = a.subtitle.startsWith('/') ? a.subtitle : `/${a.subtitle}`
    return `/subtitle/${id}${clean}`
})

const previewTitle = computed(() =>
    (previewAsset.value?.title || '').replace(/\.ts$/i, '')
)

const previewExportUrl = computed(() => {
    if (!previewAsset.value) return ''
    const id = configStore.channels[configStore.i]?.id
    const filePath =
        previewAsset.value.process_path ||
        previewAsset.value.play_path ||
        previewAsset.value.path
    return `/file/${id}${filePath}`
})

const previewKind = computed(() => {
    const a = previewAsset.value
    if (!a) return 'none'
    if (a.category === 'subtitles') return 'none'
    if (a.category === 'audio' || a.media_type === 'audio') return 'audio'
    if (a.media_type === 'image') return 'image'
    const ext = (a.title.split('.').slice(-1)[0] || '').toLowerCase()
    if (['png', 'jpg', 'jpeg', 'gif', 'webp', 'bmp'].includes(ext)) return 'image'
    return 'video'
})

const previewMime = computed(() => {
    if (!previewAsset.value) return 'video/mp4'
    // .ts assets are always served through the /preview/ remux route, which
    // returns a browser-playable fragmented MP4.
    const titleCheck = previewAsset.value.title.toLowerCase()
    if (titleCheck.endsWith('.ts')) return 'video/mp4'
    const playPath = previewAsset.value.play_path || previewAsset.value.path
    // The /preview/ remux route always returns a fragmented MP4.
    if (playPath.includes('/preview/')) return 'video/mp4'
    // Check the play_path extension first (e.g. a ZIP whose play_path points
    // at a generated .webm preview).
    const playExt = (playPath.split('.').pop() || '').toLowerCase()
    if (playExt === 'webm') return 'video/webm'
    if (playExt === 'mov') return 'video/quicktime'
    if (playExt === 'mp4' || playExt === 'm4v') return 'video/mp4'
    if (playExt === 'mkv') return 'video/x-matroska'
    const title = previewAsset.value.title.toLowerCase()
    if (title.endsWith('.webm')) return 'video/webm'
    if (title.endsWith('.mov')) return 'video/quicktime'
    if (title.endsWith('.mp4') || title.endsWith('.m4v')) return 'video/mp4'
    if (title.endsWith('.ts')) return 'video/mp4'
    if (title.endsWith('.mkv')) return 'video/x-matroska'
    return 'video/mp4'
})

const probe = computed(() => previewAsset.value?.probe)

const videoStreamInfo = computed(() => {
    const p = probe.value
    if (!p || !p.width || !p.height) {
        return { streams: 0, entries: [] as ProbeEntry[] }
    }
    const fps = previewAsset.value?.frame_rate
    const entries: ProbeEntry[] = []
    if (p.codec) entries.push({ label: 'Codec', value: p.codec })
    if (p.profile) entries.push({ label: 'Codec Profile', value: p.profile })
    entries.push({ label: 'Resolution', value: `${p.width}x${p.height}` })
    if (fps) entries.push({ label: 'Frame Frequency', value: `${fps}fps` })
    if (p.display_aspect_ratio) entries.push({ label: 'Aspect Ratio', value: aspectLabel(p.display_aspect_ratio) })
    const scan = scanTypeLabel(p.field_order)
    if (scan) entries.push({ label: 'Scan Type', value: scan })
    return { streams: 1, entries }
})

const audioStreamInfo = computed(() => probe.value?.audio ?? [])

const subtitleStreamInfo = computed(() => {
    const entries: MediaProbeStream[] = [...(probe.value?.subtitles ?? [])]
    const a = previewAsset.value
    // Closed-caption sidecar (.srt/.vtt etc.) next to the processed video.
    // ffprobe does not report these as subtitle streams, so surface them so
    // the Video/Audio/Subtitles details tab is not empty for CC assets.
    if (a?.cc) {
        const sidecarName = a.subtitle ? a.subtitle.split('/').pop() || 'Closed Captions' : 'Closed Captions'
        const ext = a.subtitle ? (a.subtitle.split('.').pop() || '') : ''
        entries.push({
            codec_type: 'subtitle',
            codec_name: ext === 'srt' ? 'subrip' : ext,
            tags: { title: sidecarName },
        })
    }
    return entries
})

const fileSize = computed(() => formatBytes(probe.value?.size || 0))

const bitrate = computed(() => formatBitrate(probe.value?.bit_rate || 0))

const tcInSeconds = computed(() => {
    const a = previewAsset.value
    if (a && a.tc_in != null) return a.tc_in
    return probe.value?.start_time || 0
})

const tcIn = computed(() => {
    if (tcInOverride.value) return tcInOverride.value
    const a = previewAsset.value
    if (a && a.tc_in != null) return smpteTime(a.tc_in, parseFps(a.frame_rate))
    const p = probe.value
    if (!p || !p.size) return '—'
    return smpteTime(p.start_time || 0, parseFps(a?.frame_rate))
})

const tcOut = computed(() => {
    if (tcOutOverride.value) return tcOutOverride.value
    const a = previewAsset.value
    if (a && a.tc_out != null) return smpteTime(a.tc_out, parseFps(a.frame_rate))
    const dur = a?.duration_seconds || 0
    if (!dur) return '—'
    return smpteTime(tcInSeconds.value + dur, parseFps(a?.frame_rate))
})

const blockLengthSeconds = computed(() => {
    const a = previewAsset.value
    const total = a?.duration_seconds || 0
    if (!a || !total) return 0
    if (a.segment_length && a.segment_length > 0) return a.segment_length
    return a.segments ? total / a.segments : 0
})

const segmentLength = computed(() => Math.round(blockLengthSeconds.value))

const isVideoPreview = computed(() => previewKind.value === 'video')

const effectiveSegments = computed(() => {
    const a = previewAsset.value
    const total = a?.duration_seconds || 0
    const length = blockLengthSeconds.value
    if (!a || !total || !length) return [] as { start: number; end: number }[]
    if (segmentsCustom.value && segmentsCustom.value.length) return segmentsCustom.value
    const count = Math.max(1, Math.ceil(total / length))
    const segs: { start: number; end: number }[] = []
    for (let i = 0; i < count; i++) {
        const start = i * length
        segs.push({ start, end: Math.min(start + length, total) })
    }
    return segs
})

const segmentBlocks = computed(() => {
    const a = previewAsset.value
    const total = a?.duration_seconds || 0
    if (!a || !total) {
        return [] as { idx: number; width: number; label: string }[]
    }
    const fps = parseFps(a.frame_rate)
    return effectiveSegments.value.map((s, i) => ({
        idx: i,
        width: (Math.max(0, s.end - s.start) / total) * 100,
        label: smpteTime(s.start, fps),
    }))
})

const segmentTableRows = computed(() => {
    const a = previewAsset.value
    const total = a?.duration_seconds || 0
    if (!a || !total) {
        return [] as {
            index: number
            start: string
            end: string
            duration: string
            start_sec: number
            end_sec: number
        }[]
    }
    const fps = parseFps(a.frame_rate)
    return effectiveSegments.value.map((s, i) => ({
        index: i,
        start: smpteTime(s.start, fps),
        end: smpteTime(s.end, fps),
        duration: smpteTime(Math.max(0, s.end - s.start), fps),
        start_sec: s.start,
        end_sec: s.end,
    }))
})

const qcRows = computed(() => {
    const a = previewAsset.value
    if (!a) return [] as ProbeEntry[]
    const v = videoStreamInfo.value
    const rows: ProbeEntry[] = []
    const res = v.entries.find((e) => e.label === 'Resolution')
    const fps = a.frame_rate
    const codec = v.entries.find((e) => e.label === 'Codec')
    if (codec) rows.push({ label: 'Codec', value: codec.value })
    if (res) rows.push({ label: 'Resolution', value: res.value })
    if (fps) rows.push({ label: 'Frame Rate', value: fps })
    rows.push({ label: 'Audio Streams', value: String(a.audio) })
    rows.push({ label: 'Closed Captions', value: a.cc ? 'Yes' : 'No' })
    rows.push({ label: 'Overall Status', value: a.status })
    return rows
})

const metaRows = computed(() => {
    const a = previewAsset.value
    if (!a) return [] as ProbeEntry[]
    return [
        { label: 'Title', value: a.title },
        { label: 'Asset ID', value: a.asset_id },
        { label: 'Category', value: a.category },
        { label: 'Type', value: a.media_type },
        { label: 'Duration', value: a.duration },
        { label: 'Segments', value: String(a.segments) },
        { label: 'Audio', value: String(a.audio) },
        { label: 'Closed Captions', value: `CC ${a.cc ? 1 : 0}` },
        { label: 'Frame Rate', value: a.frame_rate },
        { label: 'File Size', value: fileSize.value },
        { label: 'Bitrate', value: bitrate.value },
    ]
})

const LANGUAGE_NAMES: Record<string, string> = {
    ara: 'Arabic',
    eng: 'English',
    fra: 'French',
    ger: 'German',
    deu: 'German',
    spa: 'Spanish',
    hin: 'Hindi',
    ben: 'Bengali',
    urd: 'Urdu',
    tam: 'Tamil',
    tel: 'Telugu',
    mar: 'Marathi',
    kan: 'Kannada',
    guj: 'Gujarati',
    mal: 'Malayalam',
    pun: 'Punjabi',
    por: 'Portuguese',
    ita: 'Italian',
    rus: 'Russian',
    chi: 'Chinese',
    zho: 'Chinese',
    jpn: 'Japanese',
    kor: 'Korean',
    tur: 'Turkish',
    dut: 'Dutch',
    nld: 'Dutch',
    swe: 'Swedish',
    dan: 'Danish',
    nor: 'Norwegian',
    fin: 'Finnish',
    pol: 'Polish',
    ces: 'Czech',
    hun: 'Hungarian',
    rum: 'Romanian',
    ukr: 'Ukrainian',
    gre: 'Greek',
    ell: 'Greek',
    heb: 'Hebrew',
    tha: 'Thai',
    vie: 'Vietnamese',
    ind: 'Indonesian',
    fas: 'Persian',
    per: 'Persian',
}

function streamLang(stream: MediaProbeStream): string {
    return (stream.tags?.language || stream.tags?.Language || '').toLowerCase()
}

function streamLabel(stream: MediaProbeStream, index: number): string {
    const title = stream.tags?.title
    if (title) return title
    const lang = streamLang(stream)
    if (lang) {
        const name = LANGUAGE_NAMES[lang]
        return name
            ? `${name} (${lang.toUpperCase()}) (${lang})`
            : `Stream ${index + 1} (${lang.toUpperCase()})`
    }
    return `Stream ${index + 1}`
}

function streamCodecLabel(stream: MediaProbeStream): string {
    return stream.codec_name ? stream.codec_name.toUpperCase() : ''
}

function aspectLabel(dar?: string): string {
    if (!dar) return ''
    return dar === '0:1' ? '' : dar
}

function scanTypeLabel(fieldOrder?: string): string {
    if (!fieldOrder) return ''
    return fieldOrder === 'progressive' ? 'progressive' : 'interlaced'
}

function parseFps(fr?: string): number {
    const v = parseFloat(fr || '')
    return isFinite(v) && v > 0 ? v : 25
}

function pad(n: number): string {
    return String(n).padStart(2, '0')
}

function smpteTime(seconds: number, fps: number): string {
    const r = Math.max(1, Math.round(fps))
    const total = Math.max(0, Math.round(seconds * r))
    const ff = total % r
    const secsTotal = Math.floor(total / r)
    return `${pad(Math.floor(secsTotal / 3600))}:${pad(Math.floor(secsTotal / 60) % 60)}:${pad(secsTotal % 60)}:${pad(ff)}`
}

function formatBytes(bytes: number): string {
    if (!bytes || bytes <= 0) return '—'
    const units = ['B', 'KB', 'MB', 'GB', 'TB']
    let v = bytes
    let u = 0
    while (v >= 1024 && u < units.length - 1) {
        v /= 1024
        u++
    }
    return `${v.toFixed(u ? 2 : 0)} ${units[u]}`
}

function formatBitrate(bps: number): string {
    if (!bps || bps <= 0) return '—'
    return `${(bps / 1_000_000).toFixed(2)} Mbps`
}

function resetSegmentState() {
    segmentTimecode.value = '00:00:00:00'
    currentSegment.value = 0
    segmentPlaying.value = false
    segmentsCustom.value = null
    editingSegment.value = -1
    editSegStart.value = ''
    editSegEnd.value = ''
}

function seekToSegment(index: number) {
    if (!isVideoPreview.value) return
    const v = previewVideoRef.value
    const rows = segmentTableRows.value
    if (!v || !rows.length) return
    const idx = Math.min(Math.max(index, 0), rows.length - 1)
    const row = rows[idx]
    if (!row) return
    v.currentTime = row.start_sec
    currentSegment.value = idx
}

function seekSegment(delta: number) {
    seekToSegment(currentSegment.value + delta)
}

function toggleSegmentPlay() {
    const v = previewVideoRef.value
    if (!v) return
    if (v.paused) {
        v.play()
    } else {
        v.pause()
    }
}

function parseTimecode(tc: string): number {
    const parts = tc.split(':').map((p) => parseInt(p, 10))
    if (parts.some((p) => isNaN(p))) return NaN
    const fps = parseFps(previewAsset.value?.frame_rate)
    return (
        (parts[0] || 0) * 3600 +
        (parts[1] || 0) * 60 +
        (parts[2] || 0) +
        ((parts[3] || 0) / fps)
    )
}

function seekToTimecode() {
    if (!isVideoPreview.value) return
    const v = previewVideoRef.value
    if (!v) return
    const secs = parseTimecode(segmentTimecode.value)
    if (isNaN(secs)) return
    const max = v.duration || previewAsset.value?.duration_seconds || 0
    v.currentTime = Math.max(0, Math.min(secs, max))
}

function onSegmentTimeupdate() {
    const v = previewVideoRef.value
    if (!v) return
    segmentTimecode.value = smpteTime(v.currentTime, parseFps(previewAsset.value?.frame_rate))
    const rows = segmentTableRows.value
    if (!rows.length) return
    let idx = rows.findIndex(
        (r) => v.currentTime >= r.start_sec && v.currentTime < r.end_sec
    )
    if (idx < 0) {
        const len = blockLengthSeconds.value
        idx = len
            ? Math.floor(v.currentTime / len)
            : rows.length - 1
    }
    currentSegment.value = Math.min(rows.length - 1, Math.max(0, idx))
}

function showPreview(asset: MediaAsset) {
    previewAsset.value = asset
    isReprocessing.value = false
    activeTab.value = 'streams'
    resetSegmentState()
    loadPreviewSegments()
    nextTick(() => {
        previewModalRef.value?.showModal()
    })
}

function openEdit() {
    const a = previewAsset.value
    if (!a) return
    editName.value = a.title
    showEdit.value = true
}

async function handleEdit(ok: boolean) {
    if (!ok) {
        showEdit.value = false
        return
    }
    const name = editName.value.trim()
    if (!name) {
        indexStore.msgAlert('warning', 'Filename cannot be empty', 3)
        return
    }
    showEdit.value = false
    if (name !== previewAsset.value?.title) {
        await renameAsset(name)
    }
}

async function renameAsset(newName: string): Promise<boolean> {
    const a = previewAsset.value
    const id = configStore.channels[configStore.i]?.id
    if (!a || !id || !newName || newName === a.title) return false
    const idx = a.path.lastIndexOf('/')
    const dst = (idx >= 0 ? a.path.slice(0, idx + 1) : '/') + newName
    const res = await fetch(`/api/file/${id}/rename/`, {
        method: 'POST',
        headers: { ...configStore.contentType, ...authStore.authHeader },
        body: JSON.stringify({ src: a.path, dst }),
    })
    if (res.status === 200) {
        indexStore.msgAlert('success', 'Asset renamed', 3)
        a.title = newName
        a.path = dst
        a.asset_id = newName.replace(/\.[^/.]+$/, '')
        if (a.play_path) {
            a.play_path = a.play_path.replace(/[^/]+$/, newName)
        }
        editingField.value = null
        await mediaStore.getLibrary()
        return true
    }
    indexStore.msgAlert('error', `Rename failed: ${await res.text()}`, 5)
    return false
}

function openFieldEdit(field: EditableField) {
    const a = previewAsset.value
    if (!a) return
    editValue.value =
        field === 'title'
            ? a.title
            : field === 'asset_id'
              ? a.asset_id
              : field === 'tc_out'
                ? tcOut.value === '—'
                    ? '00:00:00:00'
                    : tcOut.value
                : tcIn.value === '—'
                  ? '00:00:00:00'
                  : tcIn.value
    editingField.value = field
}

function cancelFieldEdit() {
    editingField.value = null
    editValue.value = ''
}

async function saveFieldEdit() {
    const a = previewAsset.value
    const field = editingField.value
    if (!a || !field) return
    const val = editValue.value.trim()
    editingField.value = null
    if (!val) return
    if (field === 'title') {
        await renameAsset(val)
    } else if (field === 'asset_id') {
        const ext = a.title.includes('.') ? a.title.slice(a.title.lastIndexOf('.')) : ''
        await renameAsset(val + ext)
    } else if (field === 'tc' || field === 'tc_out') {
        await saveAssetTc(field === 'tc' ? 'in' : 'out', val)
    }
}

async function saveAssetTc(kind: 'in' | 'out', val: string) {
    const a = previewAsset.value
    const channelId = configStore.channels[configStore.i]?.id
    if (!a || channelId == null) return
    const sec = parseTimecode(val)
    if (Number.isNaN(sec)) {
        indexStore.msgAlert('error', 'Invalid timecode', 3)
        return
    }
    const fps = parseFps(a.frame_rate)
    const inSec =
        kind === 'in'
            ? Math.max(0, sec)
            : (a.tc_in != null ? a.tc_in : tcInSeconds.value)
    const outSec =
        kind === 'out'
            ? Math.max(inSec, sec)
            : (a.tc_out != null ? a.tc_out : inSec + (a.duration_seconds || 0))
    try {
        const res = await fetch(`/api/media/tc/${channelId}/${encodeURIComponent(a.asset_id)}/`, {
            method: 'POST',
            headers: { ...configStore.contentType, ...authStore.authHeader },
            body: JSON.stringify({ in: inSec, out: outSec }),
        })
        if (!res.ok) throw new Error('TC update failed')
        tcInOverride.value = smpteTime(inSec, fps)
        tcOutOverride.value = smpteTime(outSec, fps)
        indexStore.msgAlert('success', `${kind === 'in' ? 'TC IN' : 'TC OUT'} updated (SOM/EOM in playlist)`, 3)
        mediaStore.getLibrary()
    } catch {
        indexStore.msgAlert('error', 'Could not save timecode', 3)
    }
}

function ensureCustomSegments() {
    if (!segmentsCustom.value) {
        segmentsCustom.value = effectiveSegments.value.map((s) => ({ start: s.start, end: s.end }))
    }
}

async function loadPreviewSegments() {
    const a = previewAsset.value
    const channelId = configStore.channels[configStore.i]?.id
    if (!a || channelId == null) return
    segmentsLoading.value = true
    try {
        const res = await fetch(`/api/media/segments/${channelId}/${encodeURIComponent(a.asset_id)}`, {
            headers: authStore.authHeader,
        })
        if (res.ok) {
            const data = await res.json()
            segmentsCustom.value =
                data.custom && data.segments?.length
                    ? data.segments.map((s: { start: number; end: number }) => ({ start: s.start, end: s.end }))
                    : null
        }
    } catch {
        segmentsCustom.value = null
    } finally {
        segmentsLoading.value = false
    }
}

async function persistSegments() {
    const a = previewAsset.value
    const channelId = configStore.channels[configStore.i]?.id
    if (!a || channelId == null) return
    const segs = segmentsCustom.value || effectiveSegments.value
    segmentsSaving.value = true
    try {
        const res = await fetch(`/api/media/segments/${channelId}/${encodeURIComponent(a.asset_id)}/`, {
            method: 'POST',
            headers: { ...configStore.contentType, ...authStore.authHeader },
            body: JSON.stringify({ segments: segs.map((s) => ({ start: s.start, end: s.end })) }),
        })
        if (!res.ok) throw new Error('save failed')
        indexStore.msgAlert('success', 'Segments saved', 2)
        mediaStore.getLibrary()
    } catch {
        indexStore.msgAlert('error', 'Could not save segments', 3)
    } finally {
        segmentsSaving.value = false
    }
}

function openSegmentEdit(i: number) {
    const row = segmentTableRows.value[i]
    if (!row) return
    editSegStart.value = row.start
    editSegEnd.value = row.end
    editingSegment.value = i
}

function cancelSegmentEdit() {
    editingSegment.value = -1
    editSegStart.value = ''
    editSegEnd.value = ''
}

function saveSegmentEdit(i: number) {
    const a = previewAsset.value
    const row = segmentTableRows.value[i]
    if (!a || !row) return
    const total = a.total_duration_seconds || a.duration_seconds || 0
    const s = parseTimecode(editSegStart.value)
    const e = parseTimecode(editSegEnd.value)
    if (Number.isNaN(s) || Number.isNaN(e)) {
        indexStore.msgAlert('error', 'Invalid timecode', 3)
        return
    }
    const start = Math.max(0, Math.min(s, total))
    const end = Math.max(start, Math.min(e, total))
    if (end <= start) {
        indexStore.msgAlert('error', 'End must be after Start', 3)
        return
    }
    ensureCustomSegments()
    if (!segmentsCustom.value || !segmentsCustom.value[i]) return
    segmentsCustom.value[i] = { start, end }
    editingSegment.value = -1
    indexStore.msgAlert('success', `Segment ${i + 1} updated`, 3)
    persistSegments()
}

function deleteSegment(i: number) {
    const row = segmentTableRows.value[i]
    if (!row) return
    ensureCustomSegments()
    if (!segmentsCustom.value) return
    segmentsCustom.value.splice(i, 1)
    if (editingSegment.value >= 0) {
        editingSegment.value = -1
        editSegStart.value = ''
        editSegEnd.value = ''
    }
    if (currentSegment.value >= segmentTableRows.value.length) {
        currentSegment.value = Math.max(0, segmentTableRows.value.length - 1)
    }
    indexStore.msgAlert('success', `Segment ${i + 1} deleted`, 3)
    persistSegments()
}

function createSegment() {
    const a = previewAsset.value
    if (!a || !isVideoPreview.value) return
    const v = previewVideoRef.value
    let t = v && !isNaN(v.currentTime) ? v.currentTime : parseTimecode(segmentTimecode.value)
    if (isNaN(t)) return
    const rows = segmentTableRows.value
    if (!rows.length) return
    let idx = rows.findIndex((r) => t > r.start_sec && t < r.end_sec)
    if (idx < 0) idx = 0
    ensureCustomSegments()
    if (!segmentsCustom.value) return
    const seg = segmentsCustom.value[idx]
    if (!seg) return
    const cut = Math.max(seg.start, Math.min(t, seg.end))
    if (cut <= seg.start || cut >= seg.end) {
        indexStore.msgAlert('warning', 'Could not split: outside segment bounds', 3)
        return
    }
    segmentsCustom.value.splice(idx, 1, { start: seg.start, end: cut }, { start: cut, end: seg.end })
    currentSegment.value = idx
    indexStore.msgAlert('success', `Segment ${idx + 1} split at ${smpteTime(cut, parseFps(a.frame_rate))}`, 3)
    persistSegments()
}

function closePreview() {
    previewModalRef.value?.close()
}

function onPreviewClose() {
    if (previewVideoRef.value) {
        previewVideoRef.value.pause()
        previewVideoRef.value.src = ''
    }
    previewAsset.value = null
    isReprocessing.value = false
    activeTab.value = 'streams'
    resetSegmentState()
    editingField.value = null
    editValue.value = ''
    editingSegment.value = -1
    editSegStart.value = ''
    editSegEnd.value = ''
    tcInOverride.value = ''
    tcOutOverride.value = ''
    segmentsCustom.value = null
    segmentsLoading.value = false
    segmentsSaving.value = false
}

async function reprocessPreview() {
    const a = previewAsset.value
    if (!a || isReprocessing.value) return

    isReprocessing.value = true
    const oldPath = a.path
    await mediaStore.reprocessLibrary([oldPath])
    isReprocessing.value = false

    const updated = mediaStore.library.find(
        (x) => x.play_path === oldPath || x.path === oldPath
    )
    if (updated) {
        previewAsset.value = updated
    }
}

async function deletePreview() {
    const a = previewAsset.value
    const id = configStore.channels[configStore.i]?.id
    if (!a || !id) return
    const res = await fetch(`/api/file/${id}/remove/`, {
        method: 'POST',
        headers: { ...configStore.contentType, ...authStore.authHeader },
        body: JSON.stringify({
            src: a.path,
        }),
    })
    if (res.status === 200) {
        indexStore.msgAlert('success', t('media.deleteSuccess'), 3)
        closePreview()
        await mediaStore.getLibrary()
    } else {
        indexStore.msgAlert('error', t('media.reprocessError'), 3)
    }
}
</script>
