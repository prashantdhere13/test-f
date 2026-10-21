<template>
    <div class="max-w-[1200px] xs:pe-8">
        <h2 class="pt-3 text-3xl">{{ t('config.playoutConf') }}</h2>
        <form
            v-if="configStore.playout"
            class="mt-10 grid md:grid-cols-[180px_auto] gap-5"
            @submit.prevent="onSubmitPlayout"
        >
            <div class="text-xl pt-3 md:text-right">{{ t('config.general') }}:</div>
            <div class="md:pt-4">
                <label class="form-control mb-2">
                    <div class="whitespace-pre-line">
                        {{ t('config.generalHelp') }}
                    </div>
                </label>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Stop Threshold</legend>
                    <input
                        v-model="configStore.playout.general.stop_threshold"
                        type="number"
                        min="3"
                        class="input input-sm w-full max-w-36"
                    />
                    <p class="fieldset-label items-baseline">
                        {{ t('config.stopThreshold') }}
                    </p>
                </fieldset>
            </div>

            <div class="text-xl pt-3 md:text-right">{{ t('config.processing') }}:</div>
            <div class="md:pt-4">
                <label class="form-control mb-2">
                    <div class="whitespace-pre-line">
                        {{ t('config.processingHelp') }}
                    </div>
                </label>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Mode</legend>
                    <select v-model="configStore.playout.processing.mode" class="select select-sm w-full max-w-xs">
                        <option value=""></option>
                        <option v-for="mode in processingMode" :key="mode" :value="mode">{{ mode }}</option>
                    </select>
                </fieldset>

                <fieldset class="fieldset mt-2 rounded-box w-full">
                    <label class="fieldset-label text-base-content">
                        <input v-model="configStore.playout.processing.audio_only" type="checkbox" class="checkbox" />
                        Audio Only
                    </label>
                </fieldset>
                <fieldset class="fieldset mt-2 rounded-box w-full">
                    <label class="fieldset-label text-base-content">
                        <input v-model="configStore.playout.processing.copy_audio" type="checkbox" class="checkbox" />
                        Copy Audio
                    </label>
                </fieldset>
                <fieldset class="fieldset mt-2 rounded-box w-full">
                    <label class="fieldset-label text-base-content">
                        <input v-model="configStore.playout.processing.copy_video" type="checkbox" class="checkbox" />
                        Copy Video
                    </label>
                </fieldset>

                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Width</legend>
                    <input
                        v-model="configStore.playout.processing.width"
                        type="number"
                        min="-1"
                        step="1"
                        class="input input-sm w-full max-w-36"
                    />
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Height</legend>
                    <input
                        v-model="configStore.playout.processing.height"
                        type="number"
                        min="-1"
                        step="1"
                        class="input input-sm w-full max-w-36"
                    />
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Aspect</legend>
                    <input
                        v-model="configStore.playout.processing.aspect"
                        type="number"
                        min="1"
                        step="0.001"
                        class="input input-sm w-full max-w-36"
                    />
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">FPS</legend>
                    <input
                        v-model="configStore.playout.processing.fps"
                        type="number"
                        min="1"
                        step="0.01"
                        class="input input-sm w-full max-w-36"
                    />
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">{{ t('config.processingScanType') }}</legend>
                    <select
                        v-model="configStore.playout.processing.scan_type"
                        class="select select-sm w-full max-w-xs"
                    >
                        <option value="progressive">Progressive</option>
                        <option value="interlace">Interlace</option>
                        <option value="auto">Auto (detect from source)</option>
                    </select>
                    <p class="fieldset-label items-baseline">{{ t('config.processingScanType') }}</p>
                </fieldset>

                <fieldset class="fieldset mt-2 rounded-box w-full">
                    <label class="fieldset-label text-base-content">
                        <input v-model="configStore.playout.processing.add_logo" type="checkbox" class="checkbox" />
                        Add Logo
                    </label>
                </fieldset>

                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Logo</legend>
                    <input
                        v-model="configStore.playout.processing.logo"
                        type="text"
                        name="logo"
                        class="input input-sm w-full max-w-lg"
                    />
                    <p class="fieldset-label items-baseline">{{ t('config.processingLogoPath') }}</p>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Logo Opacity</legend>
                    <input
                        v-model="configStore.playout.processing.logo_opacity"
                        type="number"
                        min="0"
                        max="1"
                        step="0.01"
                        class="input input-sm w-full max-w-36"
                    />
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Logo Scale</legend>
                    <select
                        v-model="logoScaleMode"
                        name="logo_scale"
                        class="select select-sm w-full max-w-md"
                    >
                        <option value="small">Small (54 px)</option>
                        <option value="medium">Medium (72 px)</option>
                        <option value="large">Large (108 px)</option>
                        <option value="original">Original Size</option>
                        <option value="manual">Manual</option>
                    </select>
                    <input
                        v-if="logoScaleMode === 'manual'"
                        v-model.number="manualLogoWidth"
                        type="number"
                        min="1"
                        step="1"
                        placeholder="Width"
                        class="input input-sm w-full max-w-40 mt-2"
                    />
                    <input
                        v-if="logoScaleMode === 'manual'"
                        v-model.number="manualLogoHeight"
                        type="number"
                        min="1"
                        step="1"
                        placeholder="Height"
                        class="input input-sm w-full max-w-40 mt-2"
                    />
                    <p v-if="logoScaleMode === 'manual'" class="fieldset-label items-baseline">
                        Logo width and height in pixels
                    </p>
                    <p v-else class="fieldset-label items-baseline">{{ t('config.processingLogoScale') }}</p>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Logo Position</legend>
                    <select
                        v-model="cornerSelect"
                        name="logo_position"
                        class="select select-sm w-full max-w-md"
                    >
                        <option value="topLeft">Top Left</option>
                        <option value="topRight">Top Right</option>
                        <option value="bottomLeft">Bottom Left</option>
                        <option value="bottomRight">Bottom Right</option>
                        <option v-if="customLogoPosition" value="custom">
                            Custom ({{ configStore.playout.processing.logo_position }})
                        </option>
                    </select>
                    <p class="fieldset-label items-baseline">{{ t('config.processingLogoPosition') }}</p>

                    <div v-if="!customLogoPosition" class="flex flex-col gap-2 mt-2">
                        <div class="flex items-center gap-2">
                            <label class="text-sm w-16">X offset</label>
                            <input
                                v-model.number="logoXOffset"
                                type="number"
                                step="1"
                                class="input input-sm w-24"
                            />
                            <label class="text-sm w-16">Y offset</label>
                            <input
                                v-model.number="logoYOffset"
                                type="number"
                                step="1"
                                class="input input-sm w-24"
                            />
                        </div>
                        <div class="flex items-center gap-2">
                            <button
                                type="button"
                                class="btn btn-sm btn-outline"
                                title="Move logo left"
                                @click="nudgeLogo('left')"
                            >
                                ◀ Left
                            </button>
                            <button
                                type="button"
                                class="btn btn-sm btn-outline"
                                title="Move logo right"
                                @click="nudgeLogo('right')"
                            >
                                ▶ Right
                            </button>
                            <button
                                type="button"
                                class="btn btn-sm btn-outline"
                                title="Move logo up"
                                @click="nudgeLogo('up')"
                            >
                                ▲ Up
                            </button>
                            <button
                                type="button"
                                class="btn btn-sm btn-outline"
                                title="Move logo down"
                                @click="nudgeLogo('down')"
                            >
                                ▼ Down
                            </button>
                            <span class="text-xs opacity-70">
                                X: {{ logoXOffset }} · Y: {{ logoYOffset }}px
                            </span>
                        </div>
                    </div>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Audio Tracks</legend>
                    <input
                        v-model="configStore.playout.processing.audio_tracks"
                        type="number"
                        min="1"
                        max="255"
                        step="1"
                        class="input input-sm w-full max-w-36"
                    />
                    <p class="fieldset-label items-baseline">{{ t('config.processingAudioTracks') }}</p>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Audio Track Index</legend>
                    <input
                        v-model="configStore.playout.processing.audio_track_index"
                        type="number"
                        min="-1"
                        max="255"
                        step="1"
                        class="input input-sm w-full max-w-36"
                    />
                    <p class="fieldset-label items-baseline">{{ t('config.processingAudioIndex') }}</p>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Audio Channels</legend>
                    <input
                        v-model="configStore.playout.processing.audio_channels"
                        type="number"
                        min="1"
                        max="255"
                        step="1"
                        class="input input-sm w-full max-w-36"
                    />
                    <p class="fieldset-label items-baseline">{{ t('config.processingAudioChannels') }}</p>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Volumen</legend>
                    <input
                        v-model="configStore.playout.processing.volume"
                        type="number"
                        min="0"
                        max="1"
                        step="0.001"
                        class="input input-sm w-full max-w-36"
                    />
                </fieldset>

                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Custom Filter</legend>
                    <textarea v-model="configStore.playout.processing.custom_filter" class="textarea w-full" rows="3" />
                    <p class="fieldset-label items-baseline">{{ t('config.processingCustomFilter') }}</p>
                </fieldset>

                <fieldset class="fieldset mt-2 rounded-box w-full">
                    <label class="fieldset-label text-base-content">
                        <input
                            v-model="configStore.playout.processing.override_filter"
                            type="checkbox"
                            class="checkbox"
                        />
                        Override custom Filter
                    </label>
                    <p class="fieldset-label items-baseline">{{ t('config.processingOverrideFilter') }}</p>
                </fieldset>

                <div v-if="configStore.playout.processing.override_filter" class="w-full py-0">
                    <span class="text-sm select-text font-bold text-orange-500">
                        {{ t('config.processingOverrideFilter') }}
                    </span>
                </div>

                <fieldset class="fieldset mt-2 rounded-box w-full">
                    <label class="fieldset-label text-base-content">
                        <input v-model="configStore.playout.processing.vtt_enable" type="checkbox" class="checkbox" />
                        Enable VTT
                    </label>
                    <p class="fieldset-label items-baseline">{{ t('config.processingVTTEnable') }}</p>
                </fieldset>

                <fieldset class="fieldset mt-2 rounded-box w-full">
                    <label class="fieldset-label text-base-content">
                        <input
                            v-model="configStore.playout.processing.burn_subtitles"
                            type="checkbox"
                            class="checkbox"
                        />
                        {{ t('config.processingBurnSubtitles') }}
                    </label>
                    <p class="fieldset-label items-baseline">{{ t('config.processingBurnSubtitlesHint') }}</p>
                </fieldset>


                <fieldset class="fieldset">
                    <legend class="fieldset-legend">VTT Dummy</legend>
                    <input
                        v-model="configStore.playout.processing.vtt_dummy"
                        type="text"
                        name="vtt_dummy"
                        class="input input-sm w-full max-w-lg"
                    />
                    <p class="fieldset-label items-baseline">{{ t('config.processingVTTDummy') }}</p>
                </fieldset>

                <fieldset class="fieldset mt-2">
                    <legend class="fieldset-legend">{{ t('config.processingSubtitle') }}</legend>
                    <input
                        v-model="configStore.playout.processing.subtitle"
                        type="text"
                        name="subtitle"
                        placeholder="subtitles/myfile.stl"
                        class="input input-sm w-full max-w-lg"
                    />
                    <p class="fieldset-label items-baseline">{{ t('config.processingSubtitleHint') }}</p>
                </fieldset>

                <div class="w-full">
                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">{{ t('config.processingSubtitleFont') }}</legend>
                        <input
                            v-model="configStore.playout.processing.subtitle_font"
                            type="text"
                            name="subtitle_font"
                            class="input input-sm w-full max-w-64"
                        />
                        <p class="fieldset-label items-baseline">{{ t('config.processingSubtitleFontHint') }}</p>
                    </fieldset>

                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">{{ t('config.processingSubtitleFontSize') }}</legend>
                        <input
                            v-model="configStore.playout.processing.subtitle_fontsize"
                            type="number"
                            min="6"
                            max="200"
                            step="1"
                            class="input input-sm w-full max-w-36"
                        />
                    </fieldset>

                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">{{ t('config.processingSubtitlePosition') }}</legend>
                        <select
                            v-model="configStore.playout.processing.subtitle_position"
                            class="select select-sm w-full max-w-64"
                        >
                            <option :value="2">{{ t('config.subtitlePosBottomCenter') }}</option>
                            <option :value="1">{{ t('config.subtitlePosBottomLeft') }}</option>
                            <option :value="3">{{ t('config.subtitlePosBottomRight') }}</option>
                            <option :value="8">{{ t('config.subtitlePosTopCenter') }}</option>
                            <option :value="7">{{ t('config.subtitlePosTopLeft') }}</option>
                            <option :value="9">{{ t('config.subtitlePosTopRight') }}</option>
                        </select>
                    </fieldset>
                </div>
            </div>

            <div v-if="configStore.playout.quality" class="text-xl pt-3 md:text-right">{{ t('config.quality') }}:</div>
            <div
                v-if="configStore.playout.quality"
                class="md:pt-4 grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-3 w-full"
            >
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">{{ t('config.qualityPreset') }}</legend>
                    <select
                        v-model="configStore.playout.quality.x264_preset"
                        class="select select-sm w-full max-w-xs"
                    >
                        <option value="">default (output param)</option>
                        <option v-for="p in ['ultrafast', 'superfast', 'veryfast', 'faster', 'fast', 'medium', 'slow', 'slower', 'veryslow']" :key="p" :value="p">
                            {{ p }}
                        </option>
                    </select>
                    <p class="fieldset-label items-baseline">{{ t('config.qualityPresetHint') }}</p>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">{{ t('config.qualityCrf') }}</legend>
                    <input
                        v-model.number="configStore.playout.quality.crf"
                        type="number"
                        min="0"
                        max="51"
                        step="1"
                        class="input input-sm w-full max-w-36"
                        :disabled="!!configStore.playout.quality.video_bitrate"
                    />
                    <p class="fieldset-label items-baseline">{{ t('config.qualityCrfHint') }}</p>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">{{ t('config.qualityVBitrate') }}</legend>
                    <input
                        v-model="configStore.playout.quality.video_bitrate"
                        type="text"
                        placeholder="e.g. 4000k"
                        class="input input-sm w-full max-w-36"
                    />
                    <p class="fieldset-label items-baseline">{{ t('config.qualityVBitrateHint') }}</p>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Maxrate</legend>
                    <input
                        v-model="configStore.playout.quality.maxrate"
                        type="text"
                        placeholder="e.g. 4500k"
                        class="input input-sm w-full max-w-36"
                        :disabled="!configStore.playout.quality.video_bitrate"
                    />
                    <p class="fieldset-label items-baseline">Max VBV bitrate (needs video bitrate)</p>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Bufsize</legend>
                    <input
                        v-model="configStore.playout.quality.bufsize"
                        type="text"
                        placeholder="e.g. 9000k"
                        class="input input-sm w-full max-w-36"
                        :disabled="!(configStore.playout.quality.video_bitrate && configStore.playout.quality.maxrate)"
                    />
                    <p class="fieldset-label items-baseline">VBV buffer size</p>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">{{ t('config.qualityABitrate') }}</legend>
                    <select
                        v-model="configStore.playout.quality.audio_bitrate"
                        class="select select-sm w-full max-w-xs"
                    >
                        <option value=""></option>
                        <option v-for="b in ['64k', '96k', '128k', '160k', '192k', '256k', '320k']" :key="b" :value="b">
                            {{ b }}
                        </option>
                    </select>
                    <p class="fieldset-label items-baseline">{{ t('config.qualityABitrateHint') }}</p>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">{{ t('config.qualitySampleRate') }}</legend>
                    <select
                        v-model.number="configStore.playout.quality.sample_rate"
                        class="select select-sm w-full max-w-xs"
                    >
                        <option value=""></option>
                        <option v-for="r in [32000, 44100, 48000]" :key="r" :value="r">
                            {{ r }} Hz
                        </option>
                    </select>
                    <p class="fieldset-label items-baseline">{{ t('config.qualitySampleRateHint') }}</p>
                </fieldset>
            </div>

            <div
                v-if="configStore.playout.secondary_events"
                class="text-xl pt-3 md:text-right"
            >
                {{ t('config.seTitle') }}:
            </div>
            <div
                v-if="configStore.playout.secondary_events"
                class="md:pt-4 grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-3 w-full"
            >
                <fieldset class="fieldset">
                    <label class="fieldset-label text-base-content">
                        <input
                            v-model="configStore.playout.secondary_events.enable"
                            type="checkbox"
                            class="checkbox"
                        />
                        {{ t('config.seEnable') }}
                    </label>
                    <p class="fieldset-label items-baseline">{{ t('config.seEnableHint') }}</p>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">{{ t('config.seNnlPosition') }}</legend>
                    <select
                        v-model="nnlCfg.position"
                        name="nnl_position"
                        class="select select-sm w-full max-w-md"
                    >
                        <option value=""></option>
                        <option value="top left">Top Left</option>
                        <option value="top right">Top Right</option>
                        <option value="bottom left">Bottom Left</option>
                        <option value="bottom right">Bottom Right</option>
                        <option v-if="customNnlPosition" :value="nnlCfg.position">
                            Custom ({{ nnlCfg.position }})
                        </option>
                    </select>
                    <p class="fieldset-label items-baseline">{{ t('config.seNnlPositionHint') }}</p>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">{{ t('config.seNnlSize') }}</legend>
                    <input
                        v-model.number="nnlCfg.size"
                        type="number"
                        min="25"
                        max="400"
                        step="5"
                        class="input input-sm w-full max-w-36"
                    />
                    <p class="fieldset-label items-baseline">{{ t('config.seNnlSizeHint') }}</p>
                </fieldset>
                <fieldset class="fieldset">
                    <label class="fieldset-label text-base-content">
                        <input
                            v-model="configStore.playout.secondary_events.graphic_enable"
                            type="checkbox"
                            class="checkbox"
                        />
                        {{ t('config.seGraphicEnable') }}
                    </label>
                    <p class="fieldset-label items-baseline">{{ t('config.seGraphicHint') }}</p>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">{{ t('config.seGraphic') }}</legend>
                    <select
                        v-model="configStore.playout.secondary_events.graphic"
                        name="graphic_overlay"
                        class="select select-sm w-full max-w-md"
                    >
                        <option value="">{{ t('config.seGraphicNone') }}</option>
                        <option
                            v-for="opt in graphicOptions"
                            :key="opt.path"
                            :value="opt.path"
                        >
                            {{ opt.label }}
                        </option>
                    </select>
                    <p class="fieldset-label items-baseline">{{ t('config.seGraphicHint') }}</p>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">{{ t('config.seNnlPosition') }}</legend>
                    <select
                        v-model="configStore.playout.secondary_events.graphic_position"
                        name="graphic_position"
                        class="select select-sm w-full max-w-md"
                    >
                        <option value=""></option>
                        <option v-for="pos in graphicPositions" :key="pos" :value="pos">
                            {{ pos }}
                        </option>
                        <option
                            v-if="graphicPositionCustom"
                            :value="configStore.playout.secondary_events.graphic_position"
                        >
                            Custom ({{ configStore.playout.secondary_events.graphic_position }})
                        </option>
                    </select>
                    <p class="fieldset-label items-baseline">{{ t('config.seNnlPositionHint') }}</p>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">{{ t('config.seGraphicScale') }}</legend>
                    <input
                        v-model.number="configStore.playout.secondary_events.graphic_scale"
                        type="number"
                        min="1"
                        max="400"
                        step="5"
                        class="input input-sm w-full max-w-36"
                    />
                    <p class="fieldset-label items-baseline">{{ t('config.seGraphicScaleHint') }}</p>
                </fieldset>
            </div>

            <div class="text-xl pt-3 md:text-right">{{ t('config.ingest') }}:</div>
            <div class="md:pt-4">
                <label class="form-control mb-2">
                    <div class="whitespace-pre-line">
                        {{ t('config.ingestHelp') }}
                    </div>
                </label>

                <fieldset class="fieldset mt-2 rounded-box w-full">
                    <label class="fieldset-label text-base-content">
                        <input v-model="configStore.playout.ingest.enable" type="checkbox" class="checkbox" />
                        Enable
                    </label>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Input Param</legend>
                    <input
                        v-model="configStore.playout.ingest.input_param"
                        type="text"
                        class="input input-sm w-full max-w-lg"
                    />
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Custom Filter</legend>
                    <textarea v-model="configStore.playout.ingest.custom_filter" class="textarea w-full" rows="3" />
                    <p class="fieldset-label items-baseline">{{ t('config.ingestCustomFilter') }}</p>
                </fieldset>
            </div>

            <div class="text-xl pt-3 md:text-right">{{ t('config.playlist') }}:</div>
            <div class="md:pt-4">
                <label class="form-control mb-2">
                    <div class="whitespace-pre-line">
                        {{ t('config.playlistHelp') }}
                    </div>
                </label>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Day Start</legend>
                    <input
                        v-model="configStore.playout.playlist.day_start"
                        type="text"
                        name="day_start"
                        class="input input-sm w-full max-w-xs"
                        pattern="(([01]?[0-9]|2[0-4]):[0-5][0-9]:[0-5][0-9]|now|none)"
                    />
                    <p class="fieldset-label items-baseline">{{ t('config.playlistDayStart') }}</p>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Length</legend>
                    <input
                        v-model="configStore.playout.playlist.length"
                        type="text"
                        name="length"
                        class="input input-sm w-full max-w-xs"
                        pattern="([01]?[0-9]|2[0-4]):[0-5][0-9]:[0-5][0-9]"
                    />
                    <p class="fieldset-label items-baseline">{{ t('config.playlistLength') }}</p>
                </fieldset>
                <fieldset class="fieldset mt-2 rounded-box w-full">
                    <label class="fieldset-label text-base-content">
                        <input v-model="configStore.playout.playlist.infinit" type="checkbox" class="checkbox" />
                        Infinit
                    </label>
                    <p class="fieldset-label items-baseline">{{ t('config.playlistInfinit') }}</p>
                </fieldset>
            </div>

            <div class="text-xl pt-3 md:text-right">{{ t('config.storage') }}:</div>
            <div class="md:pt-4">
                <label class="form-control mb-2">
                    <div class="whitespace-pre-line">
                        {{ t('config.storageHelp') }}
                    </div>
                </label>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Filler</legend>
                    <input
                        v-model="configStore.playout.storage.filler"
                        type="text"
                        name="filler"
                        class="input input-sm w-full max-w-lg"
                    />
                    <p class="fieldset-label items-baseline">{{ t('config.storageFiller') }}</p>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Extensions</legend>
                    <input v-model="extensions" type="text" name="extensions" class="input input-sm w-full max-w-lg" />
                    <p class="fieldset-label items-baseline">{{ t('config.storageExtension') }}</p>
                </fieldset>
                <fieldset class="fieldset mt-2 rounded-box w-full">
                    <label class="fieldset-label text-base-content">
                        <input v-model="configStore.playout.storage.shuffle" type="checkbox" class="checkbox" />
                        Shuffle
                    </label>
                    <p class="fieldset-label items-baseline">{{ t('config.storageShuffle') }}</p>
                </fieldset>
            </div>

            <div class="text-xl pt-3 md:text-right">{{ t('config.text') }}:</div>
            <div class="md:pt-4">
                <label class="form-control mb-2">
                    <div class="whitespace-pre-line">
                        {{ t('config.textHelp') }}
                    </div>
                </label>
                <fieldset class="fieldset mt-2 rounded-box w-full">
                    <label class="fieldset-label text-base-content">
                        <input v-model="configStore.playout.text.add_text" type="checkbox" class="checkbox" />
                        Add Text
                    </label>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Font</legend>
                    <input
                        v-model="configStore.playout.text.font"
                        type="text"
                        name="font"
                        class="input input-sm w-full max-w-lg"
                    />
                    <div class="label">
                        <span class="text-sm select-text text-base-content/80">{{ t('config.textFont') }}</span>
                    </div>
                </fieldset>

                <fieldset class="fieldset mt-2 rounded-box w-full">
                    <label class="fieldset-label text-base-content">
                        <input v-model="configStore.playout.text.text_from_filename" type="checkbox" class="checkbox" />
                        Text from File
                    </label>
                    <p class="fieldset-label items-baseline">{{ t('config.textFromFile') }}</p>
                </fieldset>

                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Style</legend>
                    <input
                        v-model="configStore.playout.text.style"
                        type="text"
                        name="style"
                        class="input input-sm w-full truncate"
                    />
                    <p class="fieldset-label items-baseline">{{ t('config.textStyle') }}</p>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Regex</legend>
                    <input
                        v-model="configStore.playout.text.regex"
                        type="text"
                        name="regex"
                        class="input input-sm w-full max-w-lg"
                    />
                    <p class="fieldset-label items-baseline">{{ t('config.textRegex') }}</p>
                </fieldset>
            </div>

            <div class="text-xl pt-3 md:text-right">{{ t('config.task') }}:</div>
            <div class="md:pt-4">
                <label class="form-control mb-2">
                    <div class="whitespace-pre-line">
                        {{ t('config.taskHelp') }}
                    </div>
                </label>

                <fieldset class="fieldset mt-2 rounded-box w-full">
                    <label class="fieldset-label text-base-content">
                        <input v-model="configStore.playout.task.enable" type="checkbox" class="checkbox" />
                        Enable
                    </label>
                </fieldset>

                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Path</legend>
                    <input
                        v-model="configStore.playout.task.path"
                        type="text"
                        name="task_path"
                        class="input input-sm w-full max-w-lg"
                    />
                    <p class="fieldset-label items-baseline">{{ t('config.taskPath') }}</p>
                </fieldset>
            </div>

            <div class="text-xl pt-3 md:text-right">{{ t('config.output') }}:</div>
            <div class="md:pt-4">
                <label class="form-control mb-2">
                    <div class="whitespace-pre-line">
                        {{ t('config.outputHelp') }}
                    </div>
                </label>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Mode</legend>
                    <select v-model="output" class="select select-sm w-full max-w-xs">
                        <option value=""></option>
                        <option v-for="output in configStore.outputs" :key="output.id" :value="output.name">{{ output.name }}</option>
                    </select>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Output Parameter</legend>
                    <textarea v-model="configStore.playout.output.output_param" class="textarea w-full" rows="6" />
                    <p class="fieldset-label items-baseline">{{ t('config.outputParam') }}</p>
                </fieldset>

                <div v-if="configStore.playout.output.mode === 'hls'" class="mt-4 mb-2">
                    <div class="text-lg font-bold mb-1">{{ t('config.abrTitle') }}</div>
                    <p class="text-sm text-base-content/70 mb-3">{{ t('config.abrHelp') }}</p>

                    <div v-if="configStore.playout.hls_variants && configStore.playout.hls_variants.length" class="overflow-x-auto mb-3">
                        <table class="table table-sm">
                            <thead>
                                <tr>
                                    <th>{{ t('config.abrName') }}</th>
                                    <th>{{ t('config.abrWidth') }}</th>
                                    <th>{{ t('config.abrHeight') }}</th>
                                    <th>{{ t('config.abrBitrate') }}</th>
                                    <th>{{ t('config.abrAudioBitrate') }}</th>
                                    <th></th>
                                </tr>
                            </thead>
                            <tbody>
                                <tr v-for="(variant, index) in configStore.playout.hls_variants" :key="index">
                                    <td>
                                        <input
                                            v-model="variant.name"
                                            type="text"
                                            class="input input-sm w-full max-w-24"
                                        />
                                    </td>
                                    <td>
                                        <input
                                            v-model.number="variant.width"
                                            type="number"
                                            min="0"
                                            step="1"
                                            class="input input-sm w-full max-w-20"
                                        />
                                    </td>
                                    <td>
                                        <input
                                            v-model.number="variant.height"
                                            type="number"
                                            min="0"
                                            step="1"
                                            class="input input-sm w-full max-w-20"
                                        />
                                    </td>
                                    <td>
                                        <input
                                            v-model="variant.bitrate"
                                            type="text"
                                            placeholder="e.g. 4000k"
                                            class="input input-sm w-full max-w-24"
                                        />
                                    </td>
                                    <td>
                                        <input
                                            v-model="variant.audio"
                                            type="text"
                                            placeholder="e.g. 128k"
                                            class="input input-sm w-full max-w-24"
                                        />
                                    </td>
                                    <td>
                                        <button
                                            class="btn btn-sm btn-error"
                                            @click="removeVariant(index)"
                                        >
                                            &times;
                                        </button>
                                    </td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                    <button class="btn btn-sm btn-primary" type="button" @click="playoutAddVariant">
                        + Add Rendition
                    </button>
                </div>

                <div class="text-lg font-bold mt-4 mb-2">MPEG-TS PIDs</div>
                <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">Video PID</legend>
                        <input
                            v-model.number="configStore.playout.output.video_pid"
                            type="number"
                            min="16"
                            max="8190"
                            class="input input-sm w-full"
                        />
                    </fieldset>
                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">Audio PID</legend>
                        <input
                            v-model.number="configStore.playout.output.audio_pid"
                            type="number"
                            min="16"
                            max="8190"
                            class="input input-sm w-full"
                        />
                    </fieldset>
                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">PMT PID</legend>
                        <input
                            v-model.number="configStore.playout.output.pmt_pid"
                            type="number"
                            min="16"
                            max="8190"
                            class="input input-sm w-full"
                        />
                    </fieldset>
                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">Service ID</legend>
                        <input
                            v-model.number="configStore.playout.output.service_id"
                            type="number"
                            min="1"
                            max="65535"
                            class="input input-sm w-full"
                        />
                    </fieldset>
                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">SCTE35 PID</legend>
                        <input
                            v-model.number="configStore.playout.output.scte_pid"
                            type="number"
                            min="16"
                            max="8190"
                            class="input input-sm w-full"
                        />
                    </fieldset>
                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">SCTE35 Marker Type</legend>
                        <select
                            v-model="configStore.playout.output.scte_marker_type"
                            class="select select-sm w-full"
                        >
                            <option value="splice_insert">Splice Insert</option>
                            <option value="time_signal">Time Signal</option>
                        </select>
                    </fieldset>
                </div>

            </div>
            <div class="mt-5 mb-10">
                <button class="btn btn-primary" type="submit">{{ t('config.save') }}</button>
            </div>
        </form>
    </div>

    <GenericModal
        :title="t('config.restartTile')"
        :text="t('config.restartText')"
        :show="configStore.showRestartModal"
        :modal-action="configStore.restart"
    />
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import GenericModal from '@/components/GenericModal.vue'

import { useAuth } from '@/stores/auth'
import { useIndex } from '@/stores/index'
import { useConfig } from '@/stores/config'
import { useMedia } from '@/stores/media'
import type { OutputMode } from '@/types/playout_config'

const { t } = useI18n()
const authStore = useAuth()
const configStore = useConfig()
const indexStore = useIndex()
const mediaStore = useMedia()

const processingMode = ['folder', 'playlist']

type LogoCorner = 'topLeft' | 'topRight' | 'bottomLeft' | 'bottomRight'

const LOGO_MARGIN = 20
const LOGO_STEP = 5

const logoCornerDefs: Record<LogoCorner, { isRight: boolean; isBottom: boolean }> = {
    topLeft: { isRight: false, isBottom: false },
    topRight: { isRight: true, isBottom: false },
    bottomLeft: { isRight: false, isBottom: true },
    bottomRight: { isRight: true, isBottom: true },
}

const logoCorner = ref<LogoCorner>('topRight')
const logoOx = ref(0)
const logoOy = ref(0)
const logoSyncing = ref(false)

function buildLogoPosition(corner: LogoCorner, ox: number, oy: number): string {
    const { isRight, isBottom } = logoCornerDefs[corner]
    // The offsets are always expressed in SCREEN coordinates: a positive X
    // moves the logo right, a positive Y moves it down, whichever corner is
    // anchored. Right/bottom anchors are measured from the far edge, so the
    // sign has to be flipped there - otherwise "move right" would subtract
    // from the right margin and shift the logo left instead.
    const term = (v: number, invert: boolean) => {
        const n = invert ? -v : v
        return n !== 0 ? (n > 0 ? `+${n}` : `${n}`) : ''
    }
    const xPart = isRight ? `W-w-${LOGO_MARGIN}${term(ox, true)}` : `${LOGO_MARGIN}${term(ox, false)}`
    const yPart = isBottom ? `H-h-${LOGO_MARGIN}${term(oy, true)}` : `${LOGO_MARGIN}${term(oy, false)}`
    return `${xPart}:${yPart}`
}

const isKnownLogoPosition = (pos: string): boolean => {
    const t = (pos || '').trim()
    if (!t) {
        return true
    }
    const [x = '', y = ''] = t.split(':')
    if (!x || !y) {
        return false
    }
    const rightX = /^W-w-20([+-]\d+)?$/.test(x)
    const leftX = /^20([+-]\d+)?$/.test(x)
    const bottomY = /^H-h-20([+-]\d+)?$/.test(y)
    const topY = /^20([+-]\d+)?$/.test(y)
    return (rightX || leftX) && (bottomY || topY)
}

function parseLogoPosition(value: string) {
    const t = (value || '').trim()
    if (!t) {
        logoCorner.value = 'topRight'
        logoOx.value = 0
        logoOy.value = 0
        return
    }
    const [x = '', y = ''] = t.split(':')
    const isRight = x.includes('W-w')
    const isBottom = y.includes('H-h')
    logoCorner.value = isRight ? (isBottom ? 'bottomRight' : 'topRight')
                               : (isBottom ? 'bottomLeft' : 'topLeft')
    // Undo the sign flip applied when building, so the stored offsets always
    // read as screen coordinates (right = +, down = +). The flip applies to
    // the axis that is measured from the far edge, hence the explicit
    // isRight / isBottom per side rather than per corner.
    const readOffset = (side: string, baseRight: string, baseLeft: string, invert: boolean): number => {
        if (side.startsWith(baseRight)) {
            const n = parseInt((side.slice(baseRight.length) || '0').replace('+', ''), 10) || 0
            return invert ? -n : n
        }
        if (side.startsWith(baseLeft)) {
            const n = parseInt((side.slice(baseLeft.length) || '0').replace('+', ''), 10) || 0
            return invert ? -n : n
        }
        return 0
    }
    logoOx.value = readOffset(x, 'W-w-20', '20', isRight)
    logoOy.value = readOffset(y, 'H-h-20', '20', isBottom)
}

function writeLogoPosition() {
    logoPosition.value = buildLogoPosition(logoCorner.value, logoOx.value, logoOy.value)
}

const logoPosition = computed<string>({
    get() {
        return configStore.playout?.processing?.logo_position || ''
    },
    set(value: string) {
        if (!configStore.playout?.processing) {
            return
        }
        logoSyncing.value = true
        configStore.playout.processing.logo_position = value
        logoSyncing.value = false
    },
})

watch(
    () => configStore.playout?.processing?.logo_position,
    (value) => {
        if (logoSyncing.value) {
            return
        }
        parseLogoPosition(value || '')
    },
    { immediate: true },
)

const cornerSelect = computed<string>({
    get() {
        return customLogoPosition.value ? 'custom' : logoCorner.value
    },
    set(value: string) {
        if (value === 'custom' || !(value in logoCornerDefs)) {
            return
        }
        logoCorner.value = value as LogoCorner
        logoOx.value = 0
        logoOy.value = 0
        writeLogoPosition()
    },
})

const logoXOffset = computed<number>({
    get() {
        return logoOx.value
    },
    set(value: number) {
        logoOx.value = Number.isFinite(value) ? value : 0
        writeLogoPosition()
    },
})

const logoYOffset = computed<number>({
    get() {
        return logoOy.value
    },
    set(value: number) {
        logoOy.value = Number.isFinite(value) ? value : 0
        writeLogoPosition()
    },
})

function nudgeLogo(direction: 'left' | 'right' | 'up' | 'down') {
    if (direction === 'up') {
        logoOy.value -= LOGO_STEP
    } else if (direction === 'down') {
        logoOy.value += LOGO_STEP
    } else if (direction === 'left') {
        logoOx.value -= LOGO_STEP
    } else {
        logoOx.value += LOGO_STEP
    }
    writeLogoPosition()
}

const customLogoPosition = computed(() => {
    const pos = configStore.playout.processing?.logo_position
    return !!pos && !isKnownLogoPosition(pos)
})

// Logo scale is offered as Small / Medium / Large / Original / Manual.
// "Original" is stored as an empty string (the backend then normalises the
// logo to its native resolution with a sane cap); the presets are stored as
// the `-1:<height>` strings the ffmpeg scale filter understands, which keeps
// the logo's aspect ratio. Manual mode collects an explicit width and height
// and stores them as `W:H`. Anything stored that does not match a preset or a
// valid W:H pair is treated as manual so a value set by an older build still
// round-trips instead of being lost.
const LOGO_SCALE_PRESETS = {
    small: '-1:54',
    medium: '-1:72',
    large: '-1:108',
    original: '',
}

function parseLogoScale(scale: unknown) {
    const parts = String(scale ?? '').split(':')
    if (parts.length !== 2) return null
    const w = Number(parts[0])
    const h = Number(parts[1])
    if (!Number.isFinite(w) || !Number.isFinite(h) || w <= 0 || h <= 0) return null
    return { w, h }
}

// Height carried by a `-1:<height>` preset value (0 when there is none). Used
// to pre-fill the manual height field, so switching from a preset to Manual
// shows the size that is currently on air instead of two blank inputs.
function logoPresetHeight(scale: unknown): number {
    const parts = String(scale ?? '').split(':')
    if (parts.length !== 2) return 0
    const h = Number(parts[1])
    return Number.isFinite(h) && h > 0 ? h : 0
}

function logoScaleModeFromValue(scale: unknown): string {
    if (!scale) return 'original'
    for (const [mode, value] of Object.entries(LOGO_SCALE_PRESETS)) {
        if (value !== '' && value === scale) return mode
    }
    return 'manual'
}

// The selected mode is held in a ref instead of being derived from the stored
// string alone: picking "Manual" does not write a value straight away (the
// operator types the size into the very inputs the choice reveals), so a
// getter that only looked at `logo_scale` would snap the select straight back
// to the current preset and the inputs would never appear.
const logoScaleManual = ref(false)
const logoScaleSyncing = ref(false)

watch(
    () => configStore.playout?.processing?.logo_scale,
    (value) => {
        if (logoScaleSyncing.value) {
            return
        }
        logoScaleManual.value = logoScaleModeFromValue(value) === 'manual'
    },
    { immediate: true },
)

const logoScaleMode = computed({
    get() {
        if (logoScaleManual.value) return 'manual'
        return logoScaleModeFromValue(configStore.playout.processing?.logo_scale)
    },
    set(mode: string) {
        logoScaleManual.value = mode === 'manual'
        if (mode === 'manual') return
        const value = LOGO_SCALE_PRESETS[mode as keyof typeof LOGO_SCALE_PRESETS]
        if (value === undefined) return
        logoScaleSyncing.value = true
        configStore.playout.processing.logo_scale = value
        logoScaleSyncing.value = false
    },
})

function setManualLogoScale(w: number, h: number) {
    logoScaleManual.value = true
    logoScaleSyncing.value = true
    configStore.playout.processing.logo_scale = `${Math.round(w)}:${Math.round(h)}`
    logoScaleSyncing.value = false
}

const manualLogoWidth = computed({
    get() {
        return parseLogoScale(configStore.playout.processing?.logo_scale)?.w ?? null
    },
    set(value) {
        const scale = configStore.playout.processing?.logo_scale
        const parsed = parseLogoScale(scale)
        const w = Number(value)
        if (!Number.isFinite(w) || w <= 0) return
        const h = parsed?.h || logoPresetHeight(scale) || Math.round(w * 0.5)
        setManualLogoScale(w, h)
    },
})

const manualLogoHeight = computed({
    get() {
        const scale = configStore.playout.processing?.logo_scale
        return parseLogoScale(scale)?.h || logoPresetHeight(scale) || null
    },
    set(value) {
        const scale = configStore.playout.processing?.logo_scale
        const parsed = parseLogoScale(scale)
        const h = Number(value)
        if (!Number.isFinite(h) || h <= 0) return
        if (parsed?.w) {
            setManualLogoScale(parsed.w, h)
            return
        }
        // no explicit width given: keep the width automatic (ffmpeg's -1)
        logoScaleManual.value = true
        logoScaleSyncing.value = true
        configStore.playout.processing.logo_scale = `-1:${Math.round(h)}`
        logoScaleSyncing.value = false
    },
})

const nnlPositions = ['top left', 'top right', 'bottom left', 'bottom right']

const nnlCfg = computed(() => {
    const se = configStore.playout.secondary_events
    if (!se) {
        return { position: 'top right', size: 100 }
    }
    if (!se.styles) {
        se.styles = {}
    }
    if (!se.styles.nnl) {
        se.styles.nnl = { position: 'top right', size: 100 }
    }
    return se.styles.nnl
})

const customNnlPosition = computed(() => {
    const pos = configStore.playout.secondary_events?.styles?.nnl?.position
    return !!pos && !nnlPositions.includes(pos)
})

const graphicPositions = ['top left', 'top right', 'bottom left', 'bottom right']

const graphicOptions = computed(() => {
    const lib = mediaStore.library
    if (!lib || !lib.length) {
        return []
    }
    return lib
        .filter((a) => a.category === 'graphics' && a.status === 'processed')
        .map((a) => ({
            path: (a.process_path || a.path).replace(/^\//, ''),
            label: a.title,
        }))
})

const graphicPositionCustom = computed(() => {
    const pos = configStore.playout.secondary_events?.graphic_position
    return !!pos && !graphicPositions.includes(pos)
})

mediaStore.getLibrary()

const extensions = computed({
    get() {
        return configStore.playout.storage.extensions.join(',')
    },

    set(value: string) {
        configStore.playout.storage.extensions = value.replace(' ', '').split(/,|;/)
    },
})

const output = computed({
    get() {
        return configStore.outputs.find(o => o.id === configStore.playout.output.id)?.name
    },
    set(value: string) {
        const output = configStore.outputs.find(o => o.name === value)
        configStore.playout.output.output_param = output?.parameters ?? ''
        configStore.playout.output.id = output?.id ?? 0
        configStore.playout.output.mode = outputModeFromParams(output?.parameters ?? '')
    },
})

function outputModeFromParams(params: string): OutputMode {
    if (params.includes('-f hls')) {
        return 'hls'
    }
    if (params.includes('-f tee') || params.includes('[f=mpegts]') || params.includes('-f mpegts') || params.includes('-f flv')) {
        return 'stream'
    }
    if (params.includes('-f null')) {
        return 'null'
    }
    return 'desktop'
}

function playoutAddVariant() {
    if (!configStore.playout.hls_variants) {
        configStore.playout.hls_variants = []
    }
    configStore.playout.hls_variants.push({
        name: 'custom',
        width: 1280,
        height: 720,
        bitrate: '2000k',
        audio: '128k',
    })
}

function removeVariant(index: number) {
    if (configStore.playout.hls_variants) {
        configStore.playout.hls_variants.splice(index, 1)
    }
}

async function onSubmitPlayout() {
    const update = await configStore.setPlayoutConfig(configStore.playout)
    configStore.onetimeInfo = true

    if (update.status === 200) {
        indexStore.msgAlert('success', t('config.updatePlayoutSuccess'), 2)

        const id = configStore.channels[configStore.i]?.id

        await fetch(`/api/control/${id}/process/`, {
            method: 'POST',
            headers: { ...configStore.contentType, ...authStore.authHeader },
            body: JSON.stringify({ command: 'status' }),
        })
            .then(async (response) => {
                if (!response.ok) {
                    throw new Error(await response.text())
                }

                return response.json()
            })
            .then(async (response) => {
                if (response === 'active') {
                    configStore.showRestartModal = true
                }

                await configStore.getPlayoutConfig()
                await configStore.getPlayoutOutputs()
            })
            .catch((e) => {
                indexStore.msgAlert('error', e.data, 3)
            })
    } else {
        indexStore.msgAlert('error', t('config.updatePlayoutFailed'), 2)
    }
}
</script>
