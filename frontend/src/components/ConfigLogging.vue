<template>
    <div class="max-w-[1200px] xs:pe-8">
        <h2 class="pt-3 text-3xl">{{ t('config.logging') }}</h2>
        <form
            v-if="configStore.playout"
            class="mt-10 grid md:grid-cols-[180px_auto] gap-5"
            @submit.prevent="onSubmitLogging"
        >
            <div class="text-xl pt-3 md:text-right">{{ t('config.logging') }}:</div>
            <div class="md:pt-4">
                <label class="form-control mb-2">
                    <div class="whitespace-pre-line">
                        {{ t('config.logHelp') }}
                    </div>
                </label>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">ffmpeg Level</legend>
                    <select v-model="configStore.playout.logging.ffmpeg_level" class="select select-sm w-full max-w-xs">
                        <option v-for="level in logLevels" :key="level" :value="level">{{ level }}</option>
                    </select>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Ingest Level</legend>
                    <select v-model="configStore.playout.logging.ingest_level" class="select select-sm w-full max-w-xs">
                        <option v-for="level in logLevels" :key="level" :value="level">{{ level }}</option>
                    </select>
                </fieldset>
                <fieldset class="fieldset mt-2 rounded-box w-full">
                    <label class="fieldset-label text-base-content">
                        <input v-model="configStore.playout.logging.detect_silence" type="checkbox" class="checkbox" />
                        Detect Silence
                    </label>
                    <p class="fieldset-label items-baseline">{{ t('config.logDetect') }}</p>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Ignore Lines</legend>
                    <input v-model="formatIgnoreLines" type="text" class="input input-sm w-full max-w-full truncate" />
                    <p class="fieldset-label items-baseline">{{ t('config.logIgnore') }}</p>
                </fieldset>
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
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import GenericModal from '@/components/GenericModal.vue'

import { useAuth } from '@/stores/auth'
import { useIndex } from '@/stores/index'
import { useConfig } from '@/stores/config'

const { t } = useI18n()
const authStore = useAuth()
const configStore = useConfig()
const indexStore = useIndex()

const logLevels = ['INFO', 'WARNING', 'ERROR']

const formatIgnoreLines = computed({
    get() {
        return configStore.playout.logging.ignore_lines.join(';')
    },

    set(value) {
        configStore.playout.logging.ignore_lines = value.split(';')
    },
})

async function onSubmitLogging() {
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
