<template>
    <div class="max-w-[1200px] xs:pe-8">
        <h2 class="pt-3 text-3xl">{{ t('config.settings') }}</h2>
        <form
            v-if="mail"
            class="mt-10 grid md:grid-cols-[180px_auto] gap-5"
            @submit.prevent="onSave"
        >
            <div class="text-xl pt-3 md:text-right">{{ t('config.mail') }}:</div>
            <div class="md:pt-4">
                <label class="form-control mb-2">
                    <div class="whitespace-pre-line">
                        {{ t('config.mailHelp') }}
                    </div>
                </label>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Subject</legend>
                    <input
                        v-model="mail.subject"
                        type="text"
                        name="subject"
                        class="input input-sm w-full max-w-lg"
                    />
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Recipient</legend>
                    <input
                        v-model="mail.recipient"
                        type="text"
                        name="recipient"
                        class="input input-sm w-full max-w-lg"
                    />
                    <p class="fieldset-label items-baseline">{{ t('config.mailRecipientHint') }}</p>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Mail Level</legend>
                    <select v-model="mail.mail_level" class="select select-sm w-full max-w-xs">
                        <option v-for="level in logLevels" :key="level" :value="level.toLowerCase()">{{ level }}</option>
                    </select>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">Interval</legend>
                    <input
                        v-model.number="mail.interval"
                        type="number"
                        min="30"
                        step="10"
                        class="input input-sm w-full max-w-36"
                    />
                    <p class="fieldset-label items-baseline">{{ t('config.mailInterval') }}</p>
                </fieldset>

                <fieldset class="fieldset mt-2 rounded-box w-full">
                    <legend class="fieldset-legend">
                        {{ t('config.mailSmtp') }}
                    </legend>
                    <div class="grid sm:grid-cols-2 gap-3 w-full">
                        <fieldset class="fieldset">
                            <legend class="fieldset-legend">SMTP Server</legend>
                            <input
                                v-model="mail.smtp_server"
                                type="text"
                                name="smtp_server"
                                class="input input-sm w-full"
                            />
                        </fieldset>
                        <fieldset class="fieldset">
                            <legend class="fieldset-legend">SMTP Port</legend>
                            <input
                                v-model.number="mail.smtp_port"
                                type="number"
                                name="smtp_port"
                                min="1"
                                class="input input-sm w-full max-w-36"
                            />
                        </fieldset>
                        <fieldset class="fieldset">
                            <legend class="fieldset-legend">SMTP User</legend>
                            <input
                                v-model="mail.smtp_user"
                                type="text"
                                name="smtp_user"
                                class="input input-sm w-full"
                            />
                        </fieldset>
                        <fieldset class="fieldset">
                            <legend class="fieldset-legend">SMTP Password</legend>
                            <input
                                v-model="mail.smtp_password"
                                type="password"
                                name="smtp_password"
                                autocomplete="new-password"
                                class="input input-sm w-full"
                                :placeholder="t('config.mailPasswordPlaceholder')"
                            />
                        </fieldset>
                    </div>
                    <label class="fieldset-label text-base-content">
                        <input v-model="mail.smtp_starttls" type="checkbox" class="checkbox" />
                        STARTTLS
                    </label>
                    <div class="flex flex-wrap items-center gap-3 mt-2">
                        <button
                            type="button"
                            class="btn btn-sm btn-outline"
                            :disabled="mailTesting || !saveAvailable"
                            @click="onTestMail"
                        >
                            {{ mailTesting ? t('config.mailTesting') : t('config.mailTest') }}
                        </button>
                        <p v-if="mailTestResult" :class="mailTestOk ? 'text-success' : 'text-error'" class="fieldset-label !m-0">
                            {{ mailTestResult }}
                        </p>
                    </div>
                </fieldset>
            </div>

            <div class="mt-5 mb-10">
                <button class="btn btn-primary" type="submit">{{ t('config.save') }}</button>
            </div>
        </form>
    </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useHead } from '@unhead/vue'

import { useAuth } from '@/stores/auth'
import { useIndex } from '@/stores/index'
import { useConfig } from '@/stores/config'

const { t } = useI18n()
const authStore = useAuth()
const configStore = useConfig()
const indexStore = useIndex()

useHead({
    title: computed(() => t('config.settings'))
})

const logLevels = ['INFO', 'WARNING', 'ERROR']

interface MailSettings {
    show: boolean
    subject: string
    recipient: string
    mail_level: string
    interval: number
    smtp_server: string
    smtp_port: number
    smtp_user: string
    smtp_password: string
    smtp_starttls: boolean
}

const emptyMail = (): MailSettings => ({
    show: false,
    subject: '',
    recipient: '',
    mail_level: 'warning',
    interval: 30,
    smtp_server: '',
    smtp_port: 465,
    smtp_user: '',
    smtp_password: '',
    smtp_starttls: false,
})

const mail = ref<MailSettings>(emptyMail())
const mailTesting = ref(false)
const mailTestResult = ref('')
const mailTestOk = ref(false)

const saveAvailable = computed(() => !!(mail.value.smtp_server && mail.value.smtp_user && mail.value.recipient))

async function loadSettings() {
    const id = configStore.channels[configStore.i]?.id
    if (!id) {
        return
    }
    try {
        const res = await fetch(`/api/mail/settings/${id}`, {
            headers: { ...configStore.contentType, ...authStore.authHeader },
        })
        if (res.ok) {
            mail.value = { ...emptyMail(), ...(await res.json()) }
        }
    } catch {
        mail.value = emptyMail()
    }
}

watch(() => configStore.i, loadSettings, { immediate: true })

async function onSave() {
    const id = configStore.channels[configStore.i]?.id
    if (!id) {
        return
    }
    const res = await fetch(`/api/mail/settings/${id}`, {
        method: 'PUT',
        headers: { ...configStore.contentType, ...authStore.authHeader },
        body: JSON.stringify(mail.value),
    })
    if (res.ok) {
        mail.value = { ...emptyMail(), ...(await res.json()) }
        indexStore.msgAlert('success', t('config.updatePlayoutSuccess'), 2)
    } else {
        indexStore.msgAlert('error', t('config.updatePlayoutFailed'), 2)
    }
}

async function onTestMail() {
    const id = configStore.channels[configStore.i]?.id
    if (!id) {
        return
    }
    mailTesting.value = true
    mailTestResult.value = ''
    try {
        const res = await fetch(`/api/mail/settings/${id}/test`, {
            method: 'POST',
            headers: { ...configStore.contentType, ...authStore.authHeader },
            body: JSON.stringify(mail.value),
        })
        const data = await res.json()
        if (res.ok) {
            mailTestOk.value = true
            mailTestResult.value = data.detail || t('config.mailTestOk')
        } else {
            mailTestOk.value = false
            mailTestResult.value = data.detail || t('config.mailTestFailed')
        }
    } catch (e: any) {
        mailTestOk.value = false
        mailTestResult.value = e?.data || String(e)
    } finally {
        mailTesting.value = false
    }
}
</script>