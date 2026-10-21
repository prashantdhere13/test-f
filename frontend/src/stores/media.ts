import { defineStore } from 'pinia'

import { useAuth } from './auth'
import { useIndex } from './index'
import { useConfig } from './config'
import { i18n } from '@/i18n'

import { playlistOperations } from '../composables/helper'

const { genUID } = playlistOperations()

export const useMedia = defineStore('media', {
    state: () => ({
        currentPath: '',
        crumbs: [] as Crumb[],
        folderTree: {} as FileFolderObject,
        folderList: {} as FolderObject,
        folderCrumbs: [] as Crumb[],
        library: [] as MediaAsset[],
        libraryLoading: false,
        storageSummary: { usedBytes: 0, files: 0 } as { usedBytes: number; files: number },
        isLoading: false,
    }),

    getters: {},
    actions: {
        async getTree(path: string, foldersOnly: boolean = false) {
            if (!foldersOnly) {
                this.isLoading = true
            }

            const authStore = useAuth()
            const configStore = useConfig()
            const indexStore = useIndex()
            const id = configStore.channels[configStore.i]?.id
            const crumbs: Crumb[] = []
            let root = '/'

            await fetch(`/api/file/${id}/browse/`, {
                method: 'POST',
                headers: { ...configStore.contentType, ...authStore.authHeader },
                body: JSON.stringify({ source: path, folders_only: foldersOnly }),
            })
                .then((response) => {
                    if (response.status === 200) {
                        return response.json()
                    } else {
                        indexStore.msgAlert('error', i18n.t('media.notExists'), 3)

                        return {
                            source: '',
                            parent: '',
                            folders: [],
                            files: [],
                        }
                    }
                })
                .then((data) => {
                    const pathStr = `${data.parent}/` + data.source
                    const pathArr = pathStr.split('/')

                    if (path && path !== '/') {
                        for (const crumb of pathArr) {
                            if (crumb === data.parent) {
                                crumbs.push({ text: crumb, path: root })
                            } else if (crumb) {
                                root += crumb + '/'
                                crumbs.push({ text: crumb, path: root })
                            }
                        }
                    } else {
                        crumbs.push({ text: data.parent, path: '' })
                    }

                    if (foldersOnly) {
                        this.folderCrumbs = crumbs
                        data.parent_folders = data.parent_folders?.map((i: any) => ({ uid: genUID(), name: i })) ?? []
                        data.folders = data.folders.map((i: any) => ({ uid: genUID(), name: i }))
                        this.folderList = data
                    } else {
                        this.currentPath = path
                        this.crumbs = crumbs
                        data.parent_folders = data.parent_folders?.map((i: any) => ({ uid: genUID(), name: i })) ?? []
                        data.folders = data.folders.map((i: any) => ({ uid: genUID(), name: i }))
                        this.folderTree = data
                    }
                })

            this.isLoading = false
        },
        async getLibrary() {
            this.libraryLoading = true

            const authStore = useAuth()
            const configStore = useConfig()
            const id = configStore.channels[configStore.i]?.id

            await fetch(`/api/media/library/${id}`, {
                method: 'GET',
                headers: { ...authStore.authHeader },
            })
                .then((response) => {
                    if (response.status === 200) {
                        return response.json()
                    }
                    return []
                })
                .then((data) => {
                    this.library = (data as MediaAsset[]) ?? []
                })
                .catch(() => {
                    this.library = []
                })

            this.libraryLoading = false
        },
        async getStorage() {
            const authStore = useAuth()
            const configStore = useConfig()
            const id = configStore.channels[configStore.i]?.id
            if (!id) return

            await fetch(`/api/media/storage/${id}`, {
                method: 'GET',
                headers: { ...authStore.authHeader },
            })
                .then((response) => (response.ok ? response.json() : null))
                .then((data) => {
                    this.storageSummary = {
                        usedBytes: (data && data.used_bytes) ?? 0,
                        files: (data && data.files) ?? 0,
                    }
                })
                .catch(() => {
                    this.storageSummary = { usedBytes: 0, files: 0 }
                })
        },
        async reprocessLibrary(sources: string[]) {
            const authStore = useAuth()
            const configStore = useConfig()
            const indexStore = useIndex()
            const id = configStore.channels[configStore.i]?.id

            // Optimistically mark the selected assets as "processing" so they
            // show up under the Processing status while ffmpeg runs.
            this.library = this.library.map((a) =>
                sources.includes(a.path) ? { ...a, status: 'processing' } : a
            )

            const res = await fetch(`/api/media/${id}/reprocess/`, {
                method: 'POST',
                headers: { ...configStore.contentType, ...authStore.authHeader },
                body: JSON.stringify({ sources }),
            })

            if (res.status === 200) {
                const results: {
                    path: string
                    play_path?: string
                    process_path?: string
                    status: string
                }[] = await res.json()
                indexStore.msgAlert(
                    results.some((r) => r.status === 'failed')
                        ? 'error'
                        : 'success',
                    results.some((r) => r.status === 'failed')
                        ? i18n.t('media.reprocessError')
                        : i18n.t('media.reprocessDone'),
                    3
                )

                await this.getLibrary()

                // Show the reprocessed assets under their process-folder path
                // (/storage/media/process/...) with the returned status.
                results.forEach((result, index) => {
                    const source = sources[index]
                    if (!source) return

                    const asset = this.library.find((a) => a.path === source)
                    if (asset && result.path) {
                        asset.path = result.path
                        if (result.play_path) asset.play_path = result.play_path
                        if (result.process_path) asset.process_path = result.process_path
                        asset.status = result.status
                        asset.title = result.path.split('/').pop() ?? asset.title
                    }
                })
            } else {
                indexStore.msgAlert('error', i18n.t('media.reprocessError'), 3)
            }
        },
    },
})
