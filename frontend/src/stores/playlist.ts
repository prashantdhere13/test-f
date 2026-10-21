import dayjs from 'dayjs'
import { differenceWith, isEqual, omit } from 'lodash-es'
import utc from 'dayjs/plugin/utc.js'
import timezone from 'dayjs/plugin/timezone.js'

import { defineStore } from 'pinia'

import { i18n } from '@/i18n'
import { playlistOperations, isLivePlaylistItem } from '@/composables/helper'
import { useAuth } from '@/stores/auth'
import { useIndex } from '@/stores/index'
import { useConfig } from '@/stores/config'

dayjs.extend(utc)
dayjs.extend(timezone)

const { processPlaylist } = playlistOperations()

let progressTicker: ReturnType<typeof setInterval> | null = null

export const usePlaylist = defineStore('playlist', {
    state: () => ({
        playlist: [] as PlaylistItem[],
        isLoading: true,
        listDate: dayjs().format('YYYY-MM-DD'),
        current: {} as PlaylistItem,
        currentIndex: 0,
        ingestRuns: false,
        elapsedSec: 0,
        shift: 0,
        playoutIsRunning: false,
        last_channel: 0,
        firstLoad: true,
        scrollToItem: false,
        missingSources: [] as string[],
    }),

    getters: {
        progressValue(state): number {
            return state.current.out > 0 ? (state.elapsedSec * 100) / state.current.out : 0
        },
    },
    actions: {
        async getPlaylist(date: string) {
            const authStore = useAuth()
            const configStore = useConfig()
            const indexStore = useIndex()
            const channel = configStore.channels[configStore.i]?.id

            await fetch(`/api/playlist/${channel}?date=${date}`, {
                method: 'GET',
                headers: authStore.authHeader,
            })
                .then(async (response) => {
                    if (!response.ok) {
                        throw new Error(await response.text())
                    }

                    return response.json()
                })
                .then((data: Playlist) => {
                    if (data.program) {
                        const programData = processPlaylist(date, data.program, false)

                        if (
                            channel === this.last_channel &&
                            this.playlist.length > 0 &&
                            programData.length > 0 &&
                            (this.playlist[0]?.date === date || configStore.playout.playlist.infinit) &&
                            differenceWith(this.playlist, programData, (a, b) => {
                                return isEqual(omit(a, ['uid']), omit(b, ['uid']))
                            }).length > 0
                        ) {
                            indexStore.msgAlert('warning', i18n.t('player.unsavedProgram'), 3)
                        } else {
                            this.playlist = programData ?? []
                        }
                    }
                })
                .catch((e) => {
                    if (e.status >= 400) {
                        indexStore.msgAlert('error', e.data, 5)
                    } else if (
                        channel === this.last_channel &&
                        this.playlist.length > 0 &&
                        this.playlist[0]?.date === date
                    ) {
                        indexStore.msgAlert('warning', i18n.t('player.unsavedProgram'), 3)
                    } else {
                        this.playlist = []
                    }
                })

            this.last_channel = channel ?? 0
        },

        async checkMissingSources() {
            const authStore = useAuth()
            const configStore = useConfig()
            const channel = configStore.channels[configStore.i]?.id
            const sources = [
                ...new Set(
                    this.playlist
                        .filter((item) => !isLivePlaylistItem(item))
                        .map((item) => item.source)
                        .filter(Boolean)
                ),
            ]

            if (!sources.length) {
                this.missingSources = []
                return
            }

            try {
                const resp = await fetch(`/api/file/${channel}/exists/`, {
                    method: 'POST',
                    headers: { ...authStore.authHeader, 'Content-Type': 'application/json' },
                    body: JSON.stringify({ sources }),
                })

                if (resp.ok) {
                    const data = await resp.json()
                    this.missingSources = data.missing ?? []
                }
            } catch {
                // on network errors keep the previous knowledge instead of
                // flagging everything red
            }
        },

        setStatus(item: PlayoutStatus) {
            this.playoutIsRunning = item.running ?? true
            this.current = item.media
            this.currentIndex = item.index
            this.elapsedSec = item.elapsed
            this.ingestRuns = item.ingest
            this.shift = item.shift

            if (this.playoutIsRunning) {
                this.startElapsedTicker()
            } else {
                this.stopElapsedTicker()
            }
        },

        startElapsedTicker() {
            if (progressTicker) {
                return
            }

            progressTicker = setInterval(() => {
                if (!this.playoutIsRunning) {
                    this.stopElapsedTicker()
                    return
                }

                if (this.current.out > 0 && this.elapsedSec < this.current.out) {
                    this.elapsedSec += 1
                }
            }, 1000)
        },

        stopElapsedTicker() {
            if (progressTicker) {
                clearInterval(progressTicker)
                progressTicker = null
            }
        },
    },
})
