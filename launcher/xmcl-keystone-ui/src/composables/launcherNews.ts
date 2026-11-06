import { injection } from '@/util/inject'
import useSWRV from 'swrv'
import { kEnvironment } from './environment'
import { kSettingsState } from './setting'
import { useSWRVConfig } from './swrvConfig'

export interface LauncherNews {
  category: string
  image: {
    url: string
    width: number
    height: number
  }
  date: string
  title: string
  description: string
  link: string
}

export function useLauncherNews() {
  const env = injection(kEnvironment)
  const { state } = injection(kSettingsState)
  const { data, error, isValidating, mutate } = useSWRV('/launcher-news', async () => {
    // zenith: XMCL's news feed (and the system info it was sent) is not used
    if (!import.meta.env.VITE_ZENITH_NEWS_URL) return [] as LauncherNews[]
    const url = new URL(import.meta.env.VITE_ZENITH_NEWS_URL as string)
    if (env.value) {
      url.searchParams.append('version', env.value.version)
      url.searchParams.append('osRelease', env.value.osRelease)
      url.searchParams.append('os', env.value.os)
      url.searchParams.append('arch', env.value.arch)
      url.searchParams.append('env', env.value.env)
      url.searchParams.append('build', env.value.build.toString())
    }
    if (state.value) {
      url.searchParams.append('locale', state.value.locale)
    }
    const resp = await fetch(url)
    if (resp.ok) {
      const result: LauncherNews[] = await resp.json()
      return result
    }
    return []
  }, useSWRVConfig())
  const news = computed(() => data.value || [] as LauncherNews[])
  return {
    news,
    isValidating,
    mutate,
    error,
  }
}
