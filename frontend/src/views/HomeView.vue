<template>
    <div class="w-full min-h-screen xs:h-full flex justify-center items-center">
        <div v-if="authStore.isLogin" class="flex flex-col justify-center items-center w-full p-5">
            <SystemStats v-if="configStore.channels.length > 0" />

            <div class="w-full flex flex-wrap justify-center gap-1 md:gap-0 md:join mt-5">
                <RouterLink to="/player" class="btn btn-neutral join-item px-2">
                    {{ t('button.player') }}
                </RouterLink>
                <RouterLink to="/media" class="btn btn-neutral join-item px-2">
                    {{ t('button.media') }}
                </RouterLink>
                <RouterLink
                    v-if="isAdmin(authStore.role) && configStore.playout?.text?.add_text && !configStore.playout?.text?.text_from_filename"
                    to="/message"
                    class="btn btn-neutral join-item px-2"
                >
                    {{ t('button.message') }}
                </RouterLink>
                <RouterLink to="/logging" class="btn btn-neutral join-item px-2">
                    {{ t('button.logging') }}
                </RouterLink>
                <div class="dropdown">
                    <div tabindex="0" role="button" class="btn btn-neutral bg-neutral/70 px-2 join-item">
                        {{ selectedLang?.name }}
                    </div>
                    <ul tabindex="0" class="dropdown-content menu bg-base-100 rounded-box z-1 w-52 p-2 shadow-sm">
                        <li v-for="lang in locales" :key="lang.code" :title="lang.name">
                            <button class="px-1 py-2 rounded" @click="changeLang(lang)">
                                {{ lang.name }}
                            </button>
                        </li>
                    </ul>
                </div>
                <RouterLink
                    v-if="isAdmin(authStore.role)"
                    to="/configure"
                    class="btn btn-neutral join-item px-2"
                    :title="t('button.configure')"
                >
                    <i class="bi bi-gear text-[17px]" />
                </RouterLink>
                <label class="join-item btn btn-neutral swap swap-rotate px-2">
                    <input
                        type="checkbox"
                        :checked="indexStore.darkMode"
                        class="focus-within:!outline-0"
                        @change="toggleTheme"
                    />
                    <i class="swap-on bi bi-brightness-high text-[18px]"></i>
                    <i class="swap-off bi bi-moon text-[18px]"></i>
                </label>
                <button class="btn btn-neutral join-item px-2" @click="logout()" :title="t('button.logout')">
                    <i class="bi bi-door-closed text-[18px]" />
                </button>
            </div>
        </div>
        <div
            v-else
            class="relative w-full min-h-screen flex items-center justify-center px-4 py-10 overflow-hidden"
        >
            <div
                class="pointer-events-none absolute inset-0"
                style="background: radial-gradient(55rem 38rem at 12% -8%, color-mix(in oklab, var(--my-accent) 16%, transparent), transparent 62%), radial-gradient(48rem 32rem at 108% 112%, color-mix(in oklab, var(--my-blue) 14%, transparent), transparent 60%)"
            ></div>

            <div class="relative w-full max-w-md">
                <div
                    class="card bg-base-100 shadow-2xl border border-base-content/10 transition-shadow duration-300 hover:shadow-[0_1.5em_5em_rgba(0,0,0,0.25)]"
                >
                    <div class="card-body p-8 sm:p-10">
                        <div class="flex flex-col items-center text-center">
                            <img
                                src="@/assets/images/ffplayout.png"
                                class="h-14 sm:h-16 w-auto drop-shadow-sm"
                                alt="Zeeplayout Logo"
                            />
                            <h1 class="mt-5 text-4xl font-bold tracking-tight text-base-content">
                                Zeeplayout
                            </h1>
                            <p class="mt-2 text-sm text-base-content/60">{{ t('home.tagline') }}</p>
                        </div>

                        <form class="mt-9 space-y-4" @submit.prevent="login">
                            <label class="input w-full flex items-center gap-2 px-4">
                                <i class="bi bi-person text-base-content/50" />
                                <input
                                    v-model="formUsername"
                                    type="text"
                                    name="username"
                                    :placeholder="t('input.username')"
                                    class="grow focus:outline-none"
                                    autocomplete="username"
                                    required
                                />
                            </label>

                            <label class="input w-full flex items-center gap-2 px-4">
                                <i class="bi bi-lock text-base-content/50" />
                                <input
                                    v-model="formPassword"
                                    type="password"
                                    name="password"
                                    :placeholder="t('input.password')"
                                    class="grow focus:outline-none"
                                    autocomplete="current-password"
                                    required
                                />
                            </label>

                            <button type="submit" class="btn btn-accent w-full mt-2">
                                <i class="bi bi-box-arrow-in-right" />
                                {{ t('button.login') }}
                            </button>

                            <div
                                v-if="showLoginError"
                                role="alert"
                                class="alert alert-error rounded-box mt-2"
                            >
                                <SvgIcon name="error" />
                                <span>{{ formError }}</span>
                            </div>
                        </form>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useHead } from '@unhead/vue'

import { locales } from '@/i18n'

import { useAuth } from '@/stores/auth'
import { useIndex } from '@/stores/index'
import { useConfig } from '@/stores/config'
import { isAdmin } from '@/composables/permissions'

import SvgIcon from '@/components/SvgIcon.vue'
import SystemStats from '@/components/SystemStats.vue'

const { locale, t } = useI18n()
const router = useRouter()
const authStore = useAuth()
const configStore = useConfig()
const indexStore = useIndex()

const selectedLang = ref()
const formError = ref('')
const showLoginError = ref(false)
const formUsername = ref('')
const formPassword = ref('')

onMounted(() => {
    selectedLang.value = locales.find((loc: any) => loc.code === locale.value)
})

useHead({
    title: computed(() => (authStore.isLogin ? 'System' : 'Login')),
})

async function login() {
    try {
        const status = await authStore.obtainToken(formUsername.value, formPassword.value)

        formUsername.value = ''
        formPassword.value = ''
        formError.value = ''

        if (status === 401 || status === 400 || status === 403) {
            formError.value = t('alert.wrongLogin')
            showLoginError.value = true

            setTimeout(() => {
                showLoginError.value = false
            }, 3000)
        }

        await configStore.configInit()

        if (authStore.isLogin && !isAdmin(authStore.role)) {
            router.push('/player')
        }
    } catch (e) {
        formError.value = e as string
    }
}

function toggleTheme() {
    indexStore.darkMode = !indexStore.darkMode

    if (indexStore.darkMode) {
        localStorage.setItem('theme', 'dark')
        // document.documentElement.setAttribute('data-theme', 'dark')
    } else {
        localStorage.setItem('theme', 'light')
        // document.documentElement.setAttribute('data-theme', 'light')
    }
}

async function logout() {
    try {
        authStore.removeToken()
    } catch (e) {
        formError.value = e as string
    }
}

async function changeLang(lang: any) {
    selectedLang.value = lang
    locale.value = lang.code
    localStorage.setItem('language', lang.code)
}
</script>
