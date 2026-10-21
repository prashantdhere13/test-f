import { cloneDeep } from 'lodash-es'
import { defineStore } from 'pinia'
import { useRouter } from 'vue-router'

import { useAuth } from './auth'
import { useIndex } from './index'
import { i18n } from '../i18n'
import type { AdvancedConfig } from '../types/advanced_config'
import { stringFormatter } from '../composables/helper'
import { isAdmin } from '../composables/permissions'

export const useConfig = defineStore('config', {
    state: () => ({
        i: 0,
        configCount: 0,
        contentType: { 'content-type': 'application/json;charset=UTF-8' },
        channels: [] as Channel[],
        channelsRaw: [] as Channel[],
        playlistLength: 86400.0,
        advanced: {} as AdvancedConfig,
        playout: {} as PlayoutConfigExt,
        outputs: [] as PlayoutOutput[],
        currentUser: 0,
        configUser: { id: 0, username: '', role_id: 3 } as User,
        timezone: 'UTC',
        onetimeInfo: true,
        showPlayer: true,
        showRestartModal: false,
    }),

    getters: {},
    actions: {
        async configInit() {
            const authStore = useAuth()
            await authStore.inspectToken()

            if (authStore.isLogin) {
                await authStore.obtainUuid()
                await this.getChannelConfig().then(async () => {
                    await this.getPlayoutConfig()
                    await this.getPlayoutOutputs()
                    await this.getUserConfig()

                    if (isAdmin(authStore.role)) {
                        await this.getAdvancedConfig()
                    }
                })
            }
        },

        logout() {
            const authStore = useAuth()
            const router = useRouter()
            localStorage.removeItem('token')
            localStorage.removeItem('refresh')
            authStore.isLogin = false

            router.push('/')
        },

        async getChannelConfig() {
            const authStore = useAuth()
            const indexStore = useIndex()

            let statusCode = 0
            await fetch('/api/channels', {
                method: 'GET',
                headers: authStore.authHeader,
            })
                .then((response) => {
                    statusCode = response.status

                    return response
                })
                .then((response) => response.json())
                .then((objs) => {
                    if (!objs[0]) {
                        this.logout()
                        throw new Error('User not found')
                    }

                    this.channels = objs
                    this.channelsRaw = cloneDeep(objs)
                    this.configCount = objs.length
                    this.timezone = objs[this.i]?.timezone || objs[0]?.timezone || 'UTC'
                })
                .catch((e) => {
                    if (statusCode === 401) {
                        this.logout()
                    }

                    this.channels = [
                        {
                            id: 1,
                            extra_extensions: '',
                            name: 'Channel 1',
                            preview_url: '',
                            public: '',
                            playlists: '',
                            storage: '',
                        },
                    ]

                    indexStore.msgAlert('error', e, 3)
                })
        },

        async getPlayoutConfig() {
            const { timeToSeconds } = stringFormatter()
            const authStore = useAuth()
            const indexStore = useIndex()
            const id = this.channels[this.i]?.id

            await fetch(`/api/playout/config/${id}`, {
                method: 'GET',
                headers: authStore.authHeader,
            })
                .then((resp) => resp.json())
                .then((data: PlayoutConfigExt) => {
                    data.playlist.startInSec = timeToSeconds(data.playlist.day_start ?? 0)
                    data.playlist.lengthInSec = timeToSeconds(data.playlist.length ?? this.playlistLength)

                    data.quality = Object.assign(
                        {
                            x264_preset: '',
                            crf: null,
                            video_bitrate: '',
                            maxrate: '',
                            bufsize: '',
                            audio_bitrate: '128k',
                            sample_rate: 48000,
                        },
                        data.quality ?? {}
                    )

                    if (data.processing) {
                        data.processing.scan_type = data.processing.scan_type || 'progressive'
                    }

                    data.hls_variants = data.hls_variants ?? [
                        { name: '1080p', width: 1920, height: 1080, bitrate: '6000k', audio: '192k' },
                        { name: '720p', width: 1280, height: 720, bitrate: '2800k', audio: '128k' },
                        { name: '480p', width: 854, height: 480, bitrate: '1400k', audio: '128k' },
                        { name: '240p', width: 426, height: 240, bitrate: '400k', audio: '64k' },
                    ]

                    data.secondary_events = Object.assign(
                        {
                            enable: true,
                            graphic_enable: false,
                            graphic: '',
                            graphic_position: 'bottom right',
                            graphic_scale: 20,
                            styles: {} as Record<string, SecondaryEventsStyle>,
                        },
                        data.secondary_events ?? {}
                    )
                    data.secondary_events.styles!.nnl = Object.assign(
                        { position: 'top right', size: 100 },
                        data.secondary_events.styles!.nnl ?? {}
                    )

                    this.playout = data
                })
                .catch(() => {
                    indexStore.msgAlert('error', i18n.t('config.noPlayoutConfig'), 3)
                })
        },

        async getPlayoutOutputs() {
            const authStore = useAuth()
            const indexStore = useIndex()
            const id = this.channels[this.i]?.id

            await fetch(`/api/playout/outputs/${id}`, {
                method: 'GET',
                headers: authStore.authHeader,
            })
                .then((resp) => resp.json())
                .then((data: PlayoutOutput[]) => {

                    this.outputs = data
                })
                .catch(() => {
                    indexStore.msgAlert('error', i18n.t('config.noPlayoutConfig'), 3)
                })
        },

        async getAdvancedConfig() {
            const authStore = useAuth()
            const indexStore = useIndex()
            const id = this.channels[this.i]?.id

            await fetch(`/api/playout/advanced/${id}`, {
                method: 'GET',
                headers: authStore.authHeader,
            })
                .then((resp) => resp.json())
                .then((data: AdvancedConfig) => {
                    this.advanced = data
                })
                .catch(() => {
                    indexStore.msgAlert('error', i18n.t('config.noAdvancedConfig'), 3)
                })
        },

        async setPlayoutConfig(obj: any) {
            const { timeToSeconds } = stringFormatter()
            const authStore = useAuth()
            const id = this.channels[this.i]?.id

            this.playlistLength = timeToSeconds(obj.playlist.length)
            this.playout.playlist.startInSec = timeToSeconds(obj.playlist.day_start)
            this.playout.playlist.lengthInSec = timeToSeconds(obj.playlist.length)

            const update = await fetch(`/api/playout/config/${id}`, {
                method: 'PUT',
                headers: { ...this.contentType, ...authStore.authHeader },
                body: JSON.stringify(obj),
            })

            return update
        },

        async setAdvancedConfig() {
            const authStore = useAuth()
            const id = this.channels[this.i]?.id

            if (this.advanced?.id > 0) {
                const update = await fetch(`/api/playout/advanced/${id}`, {
                    method: 'PUT',
                    headers: { ...this.contentType, ...authStore.authHeader },
                    body: JSON.stringify(this.advanced),
                })

                return update
            } else {
                const update = await fetch(`/api/playout/advanced/${id}/`, {
                    method: 'POST',
                    headers: { ...this.contentType, ...authStore.authHeader },
                    body: JSON.stringify(this.advanced),
                })

                return update
            }
        },

        async getUserConfig() {
            const authStore = useAuth()

            await fetch('/api/user', {
                method: 'GET',
                headers: authStore.authHeader,
            })
                .then((response) => response.json())
                .then((data) => {
                    if (data.id === 0) {
                        this.logout()
                        throw new Error('User not found')
                    }

                    this.currentUser = data.id
                    this.configUser = data
                })
        },

        async setUserConfig(obj: any) {
            const authStore = useAuth()

            const update = await fetch(`/api/user/${obj.id}`, {
                method: 'PUT',
                headers: { ...this.contentType, ...authStore.authHeader },
                body: JSON.stringify(obj),
            })

            return update
        },

        async addNewUser(user: User) {
            const authStore = useAuth()
            delete user.confirm

            const update = await fetch('/api/user/', {
                method: 'Post',
                headers: { ...this.contentType, ...authStore.authHeader },
                body: JSON.stringify(user),
            })

            return update
        },

        async restart(res: boolean) {
            if (res) {
                const authStore = useAuth()
                const indexStore = useIndex()
                const id = this.channels[this.i]?.id

                await fetch(`/api/control/${id}/process/`, {
                    method: 'POST',
                    headers: { ...this.contentType, ...authStore.authHeader },
                    body: JSON.stringify({ command: 'restart' }),
                }).catch((e) => {
                    indexStore.msgAlert('error', e.data, 3)
                })
            }

            this.showRestartModal = false
        },

        async controlStream(command: 'start' | 'stop' | 'restart') {
            const authStore = useAuth()
            const indexStore = useIndex()
            const id = this.channels[this.i]?.id

            if (!id) {
                return
            }

            // Stream control manages the streaming output (UDP/SRT mpegts in
            // stream mode, or the HLS ABR ladder in hls mode) but never the
            // playout schedule or HLS monitoring preview, so use the dedicated
            // stream_* commands on the /process/ endpoint.
            const streamCommand = `stream_${command}`

            await fetch(`/api/control/${id}/process/`, {
                method: 'POST',
                headers: { ...this.contentType, ...authStore.authHeader },
                body: JSON.stringify({ command: streamCommand }),
            })
                .then(async (response) => {
                    if (!response.ok) {
                        throw new Error(await response.text())
                    }
                    return response.json()
                })
                .then((data) => {
                    const msg = typeof data === 'string' ? data : data?.detail || 'ok'
                    indexStore.msgAlert('success', String(msg), 3)
                })
                .catch((e) => {
                    indexStore.msgAlert('error', e.message || e.data, 3)
                })
        },
    },
})
