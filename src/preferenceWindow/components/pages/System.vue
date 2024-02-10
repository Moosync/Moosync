<!-- 
  System.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <div class="w-100 h-100" :key="refreshPage.toString()">
    <b-container fluid>
      <b-row no-gutters class="w-100">
        <div class="path-selector w-100">
          <CheckboxGroup :title="$t('settings.system.systemSettings.title')"
            :tooltip="$t('settings.system.systemSettings_tooltip')" :isExtension="false"
            :defaultValue="systemCheckboxValues" :onValueChange="onSystemPrefChange" :onValueFetch="onSystemPrefFetch"
            key="system" />

          <b-col v-if="showJukeboxField">
            <EditText class="mt-5 mb-3" :isExtension="false" :title="$t('settings.system.jukebox.pin')" key="jukebox_pin"
              :tooltip="$t('settings.system.jukebox.pin_tooltip')" defaultValue="" maxValue="6" :onlyNumber="true"
              type="password" />

            <CheckboxGroup :title="$t('settings.system.jukebox.optionalFields')"
              :tooltip="$t('settings.system.jukebox.optionalFields_tooltip')" :isExtension="false"
              :defaultValue="jukeboxOptionalFields" key="jukebox_optional_fields" />
          </b-col>

          <CheckboxGroup :title="$t('settings.system.queueSettings.title')" class="mt-4"
            :tooltip="$t('settings.system.queueSettings.tooltip')" :isExtension="false"
            :defaultValue="queueCheckboxValues" key="queue" />

          <CheckboxGroup :title="$t('settings.system.audioSettings.title')" class="mt-4"
            :tooltip="$t('settings.system.audioSettings_tooltip')" :isExtension="false"
            :defaultValue="audioCheckboxValues" key="audio" />

          <EditText class="mt-4" :title="$t('settings.system.gapless_skip')" :onlyNumber="true" :key="`gapless.skip`"
            :defaultValue="0" type="number" />

          <RadioCheckbox :title="$t('settings.system.volumePersistMode.title')" class="mt-4"
            :tooltip="$t('settings.system.volumePersistMode.tooltip')" :isExtension="false"
            :defaultValue="volumePersistModeCheckboxValues" :onValueChange="onVolumePersistValueChange"
            :onValueFetch="onVolumePersistValueChange" key="volumePersistMode" />

          <b-col v-if="showVolumeMapField">
            <b-container class="ml-1 mt-2" fluid>
              <b-row no-gutters class="d-flex" v-for="provider of volumeMapProviders" :key="provider.key">
                <b-col cols="4" class="align-self-center text-left">{{ provider.title }}</b-col>
                <b-col cols="8" lg="6">
                  <EditText :onlyNumber="true" :key="`clampMap.${provider.key.replaceAll('.', '_').toLowerCase()}.clamp`"
                    :defaultValue="provider.clamp" type="range" :showRangeText="true" />
                </b-col>
              </b-row>
            </b-container>
          </b-col>

          <CheckboxGroup :title="$t('settings.system.scrobble.provider_toggle.title')" class="mt-4"
            :tooltip="$t('settings.system.scrobble.provider_toggle.tooltip')" :isExtension="false"
            :defaultValue="scrobbleProviderCheckboxValues" key="scrobble.provider_toggle" />

          <RadioCheckbox :title="$t('settings.system.youtubeAlternative.title')" class="mt-4"
            :tooltip="$t('settings.system.youtubeAlternative.tooltip')" :isExtension="false"
            :defaultValue="youtubeAlternativeCheckboxValues" :onValueChange="onYoutubeAlternativesChanged"
            :onValueFetch="onYoutubeAlternativesFetched" key="youtubeAlt" />

          <b-col v-if="showYoutubeField">
            <CheckboxGroup :title="$t('settings.system.youtubeAlternative.youtube.options')" class="mt-4"
              :tooltip="$t('settings.system.youtubeAlternative.youtube.options_tooltip')" :isExtension="false"
              :defaultValue="youtubeAdvancedCheckboxValues" key="youtubeOptions" />
          </b-col>

          <b-col v-if="showInvidiousField">
            <AutoFillEditText class="mt-4" key="invidious_instance" :datalist="invidiousInstances"
              :title="$t('settings.system.youtubeAlternative.invidious.url')"
              :tooltip="$t('settings.system.youtubeAlternative.invidious.url_tooltip')"
              :onValueChange="onInvidiousInstanceChange" :onValueFetch="onInvidiousInstanceChange" />

            <b-container class="invidious-details">
              <b-row>
                <b-col>
                  {{ invidiousDetails }}
                </b-col>
              </b-row>
            </b-container>
            <CheckboxGroup :title="$t('settings.system.youtubeAlternative.invidious.options')" class="mt-4"
              :tooltip="$t('settings.system.youtubeAlternative.invidious.options_tooltip')" :isExtension="false"
              :defaultValue="invidiousAdvancedCheckboxValues" key="invidious" />
          </b-col>

          <b-col v-if="showPipedField">
            <AutoFillEditText class="mt-4" key="piped_instance" :datalist="pipedInstances"
              :title="$t('settings.system.youtubeAlternative.piped.url')"
              :tooltip="$t('settings.system.youtubeAlternative.piped.url_tooltip')" />

            <EditText class="mt-4" :isExtension="false" :title="$t('settings.system.youtubeAlternative.piped.username')"
              key="piped.username" :tooltip="$t('settings.system.youtubeAlternative.piped.username_tooltip')" />

            <EditText class="mt-2" :isExtension="false" :title="$t('settings.system.youtubeAlternative.piped.password')"
              key="piped.password" :tooltip="$t('settings.system.youtubeAlternative.piped.password_tooltip')"
              type="password" />
          </b-col>

          <CheckboxGroup :title="$t('settings.system.spotify.title')" :tooltip="$t('settings.system.spotify.tooltip')"
            class="mt-4" key="spotify.options" :defaultValue="spotifyCheckboxValues" :isExtension="false"
            :onValueChange="onSpotifyValueFetch" :onValueFetch="onSpotifyValueFetch" />

          <b-col v-if="showSpotifyUserPass">
            <EditText class="mt-4" :isExtension="false" :title="$t('settings.system.spotify.username')"
              key="spotify.username" :tooltip="$t('settings.system.spotify.username_tooltip')" />

            <EditText class="mt-2" :isExtension="false" :title="$t('settings.system.spotify.password')"
              key="spotify.password" :tooltip="$t('settings.system.spotify.password_tooltip')" type="password" />

            <CheckboxGroup :title="$t('settings.system.spotify.librespot.title')"
              :tooltip="$t('settings.system.spotify.librespot.tooltip')" class="mt-4" key="spotify.librespot.options"
              :defaultValue="librespotOptions" :isExtension="false" />
          </b-col>

          <b-row v-if="showRestartButton">
            <b-col cols="auto">
              <b-button class="create-button" @click="restartApp">{{ $t('settings.system.restartApp') }}</b-button>
            </b-col>
          </b-row>

          <CheckboxGroup :title="$t('settings.system.lyrics.title')" class="mt-4"
            :tooltip="$t('settings.system.lyrics.tooltip')" :isExtension="false" :defaultValue="lyricsCheckboxValues"
            key="lyrics_fetchers" />

          <EditText class="mt-5 mb-3" :isExtension="false" :title="$t('settings.system.zoom')" key="zoomFactor"
            :tooltip="$t('settings.system.zoom_tooltip')" :onValueChange="onZoomUpdate" defaultValue="100"
            type="number" />

          <EditText class="mt-5 mb-3" :isExtension="false" :title="$t('settings.system.spotify.client_id')"
            key="spotify.client_id" :tooltip="$t('settings.system.spotify.client_id_tooltip')"
            @tooltipClick="openSpotifyHelp" />
          <EditText :isExtension="false" :tooltip="$t('settings.system.spotify.client_secret_tooltip')"
            :title="$t('settings.system.spotify.client_secret')" key="spotify.client_secret"
            @tooltipClick="openSpotifyHelp" />

          <EditText v-if="!youtubeEnvExists" class="mt-5 mb-3" :isExtension="false"
            :title="$t('settings.system.youtube.client_id')" :tooltip="$t('settings.system.youtube.client_id_tooltip')"
            @tooltipClick="openYoutubeHelp" key="youtube.client_id" />
          <EditText v-if="!youtubeEnvExists" :isExtension="false" :title="$t('settings.system.youtube.client_secret')"
            :tooltip="$t('settings.system.youtube.client_secret_tooltip')" @tooltipClick="openYoutubeHelp"
            key="youtube.client_secret" />

          <EditText v-if="!lastfmEnvExists" class="mt-5 mb-3" :isExtension="false"
            :title="$t('settings.system.lastfm.client_id')" key="lastfm.client_id" />

          <EditText v-if="!lastfmEnvExists" :isExtension="false" :title="$t('settings.system.lastfm.client_secret')"
            key="lastfm.client_secret" />

          <Dropdown class="mt-5" :defaultValue="languageDropdown" :title="$t('settings.system.language')"
            :tooltip="$t('settings.system.language_tooltip')" key="system_language" :onValueChange="onLanguageChanged" />

          <b-button class="delete-button mt-4" @click="showClearPreferencesDisclaimer">{{
            $t('settings.system.clearPreferences')
          }}</b-button>
        </div>
      </b-row>
    </b-container>

    <b-modal no-close-on-backdrop centered size="md" id="clear-preferences-modal" hide-footer hide-header>
      <b-container class="response-container">
        <b-row no-gutters class="d-flex">
          <b-col class="title" cols="auto">{{ $t('settings.system.clear_preferences_title') }}</b-col>
        </b-row>
        <b-row>
          <b-col class="mt-4 waiting">{{ $t('settings.system.clear_preferences') }}</b-col>
        </b-row>
        <b-row>
          <b-col class="d-flex justify-content-center">
            <div @click="clearPreferences"
              class="delete-button button-grow mt-4 d-flex justify-content-center align-items-center">
              {{ $t('settings.system.clear_preferences_button') }}
            </div>
          </b-col>
        </b-row>
      </b-container>
      <CrossIcon @click="closeClearPreferencesModal" class="close-icon button-grow" />
    </b-modal>
  </div>
</template>

<script lang="ts">
type InvidiousInstances = [
  string,
  {
    api: boolean
    uri: string
    type: 'http' | 'https'
  }
][]

import { Component } from 'vue-facing-decorator'
import { Vue } from 'vue-facing-decorator'
import CheckboxGroup from '../CheckboxGroup.vue'
import EditText from '../EditText.vue'
import PreferenceHeader from '../PreferenceHeader.vue'
import CrossIcon from '@/icons/CrossIcon.vue'
import AutoFillEditText from '../AutoFillEditText.vue'
import { InvidiousApiResources } from '@/utils/commonConstants'
import Dropdown from '../Dropdown.vue'
import { messages } from '@/utils/ui/i18n'
import RadioCheckbox from '../RadioCheckbox.vue'
import { VolumePersistMode } from '../../../utils/commonConstants'

@Component({
  components: {
    CheckboxGroup,
    RadioCheckbox,
    EditText,
    PreferenceHeader,
    AutoFillEditText,
    CrossIcon,
    Dropdown
  }
})
export default class System extends Vue {
  showRestartButton = false
  showInvidiousField = false
  showYoutubeField = false
  showPipedField = false
  showJukeboxField = false
  showSpotifyUserPass = false
  showVolumeMapField = false

  invidiousInstances: string[] = []
  invidiousDetails = ''

  pipedInstances: string[] = []

  extensions: ExtensionDetails[] = []

  async onInvidiousInstanceChange() {
    try {
      const resp = await window.SearchUtils.requestInvidious(
        InvidiousApiResources.STATS,
        { params: undefined },
        undefined,
        true
      )
      if (resp) {
        this.invidiousDetails = `Software: ${resp.software.name}:${resp.software.branch}-${resp.software.version
          }\nUsers: ${resp.usage.users.total}\nSignup: ${resp.openRegistrations ? 'Open' : 'Closed'}`
      }
    } catch (e) {
      this.invidiousDetails = this.$t('settings.system.invidiousUrlUnsupported')
    }
  }

  get languageDropdown() {
    const items = new Intl.DisplayNames(['en'], {
      type: 'language'
    })
    const languages = []
    for (const lang of Object.keys(messages)) {
      languages.push({
        key: lang,
        title: items.of(lang.replaceAll('_', '-')),
        enabled: lang === 'en_US'
      })
    }
    return languages
  }

  onLanguageChanged(key: Checkbox[]) {
    const active = key.find((val) => val.enabled) ?? this.languageDropdown[0]
    console.debug('changing locale to', active.key)
    this.$i18n.locale = active.key
    window.ThemeUtils.setLanguage(active.key)
  }

  async fetchInvidiousInstances() {
    const resp: InvidiousInstances = await (await fetch('https://api.invidious.io/instances.json')).json()
    for (const instance of resp) {
      if (typeof instance[1] === 'object' && instance[1].api && instance[1].type === 'https') {
        this.invidiousInstances.push(instance[1].uri)
      }
    }
  }

  async fetchPipedInstances() {
    const resp = await (
      await fetch('https://raw.githubusercontent.com/wiki/TeamPiped/Piped-Frontend/Instances.md')
    ).text()
    let skipped = 0
    const lines = resp.split('\n')
    this.pipedInstances = lines
      .map((line) => {
        const split = line.split('|')
        if (split.length === 5) {
          if (skipped < 2) {
            skipped++
            return
          }
          return split[1].trim()
        }
      })
      .filter((instance) => instance?.length ?? 0 > 0) as string[]
  }

  defaultSystemSettings: SystemSettings[] = []
  defaultYoutubeAlts: Checkbox[] = []

  get spotifyCheckboxValues(): Checkbox[] {
    return [
      {
        key: 'use_librespot',
        title: this.$t('settings.system.spotify.librespotEnable'),
        enabled: false
      }
    ]
  }

  get librespotOptions(): Checkbox[] {
    return [
      {
        key: 'use_spotify_canvas',
        title: this.$t('settings.system.spotify.librespot.useSpotifyCanvas'),
        enabled: true
      },
      {
        key: 'use_librespot_playback',
        title: this.$t('settings.system.spotify.librespot.useLibrespotPlayback'),
        enabled: true
      }
    ]
  }

  onSpotifyValueFetch(value: Checkbox[]) {
    this.showSpotifyUserPass = value.find((val) => val.key === 'use_librespot')?.enabled ?? false
  }

  get invidiousAdvancedCheckboxValues(): Checkbox[] {
    return [
      {
        key: 'always_proxy',
        title: this.$t('settings.system.youtubeAlternative.invidious.always_proxy'),
        enabled: true
      }
    ]
  }

  onVolumePersistValueChange(value: Checkbox[]) {
    const active = value.find((val) => val.enabled)

    this.showVolumeMapField = false
    if (active?.key === VolumePersistMode.CLAMP_MAP) {
      this.showVolumeMapField = true
    }
  }

  get volumePersistModeCheckboxValues(): Checkbox[] {
    return [
      {
        key: VolumePersistMode.SINGLE,
        title: this.$t('settings.system.volumePersistMode.noPersist'),
        enabled: true
      },
      {
        key: VolumePersistMode.SEPARATE_VOLUME_MAP,
        title: this.$t('settings.system.volumePersistMode.persistSeparateVolumes'),
        enabled: false
      },
      {
        key: VolumePersistMode.CLAMP_MAP,
        title: this.$t('settings.system.volumePersistMode.persistClampVolume'),
        enabled: false
      }
    ]
  }

  get audioCheckboxValues(): Checkbox[] {
    return [
      {
        key: 'gapless_playback',
        title: this.$t('settings.system.audioSettings.gaplessPlayback'),
        enabled: false
      },
      {
        key: 'sponsorblock',
        title: this.$t('settings.system.audioSettings.sponsorBlock'),
        enabled: false
      }
    ]
  }

  get youtubeAdvancedCheckboxValues() {
    return [
      {
        key: 'youtube_embeds',
        title: this.$t('settings.system.youtubeAlternative.youtube.useEmbeds'),
        enabled: true
      }
    ]
  }

  get youtubeAlternativeCheckboxValues(): Checkbox[] {
    return [
      {
        key: 'use_youtube',
        title: this.$t('settings.system.youtubeAlternative.useYoutube'),
        enabled: true
      },
      {
        key: 'use_invidious',
        title: this.$t('settings.system.youtubeAlternative.useInvidious'),
        enabled: false
      },
      {
        key: 'use_piped',
        title: this.$t('settings.system.youtubeAlternative.usePiped'),
        enabled: false
      }
    ]
  }

  get lyricsCheckboxValues(): Checkbox[] {
    return [
      {
        title: this.$t('settings.system.lyrics.az_lyrics'),
        key: 'az_lyrics',
        enabled: true
      },
      {
        title: this.$t('settings.system.lyrics.google_lyrics'),
        key: 'google_lyrics',
        enabled: true
      },
      {
        title: this.$t('settings.system.lyrics.spotify_lyrics'),
        key: 'spotify_lyrics',
        enabled: true
      },
      {
        title: this.$t('settings.system.lyrics.genius_lyrics'),
        key: 'genius_lyrics',
        enabled: true
      }
    ]
  }

  get systemCheckboxValues(): SystemSettings[] {
    return [
      this.startupCheckbox,
      this.minimizeToTrayCheckbox,
      this.hardwareAcceleration,
      this.watchFileChanges,
      this.enableJukeboxMode,
      this.lastLoadedPlaybackState
    ]
  }

  get lastLoadedPlaybackState(): SystemSettings {
    return {
      key: 'last_loaded_playback_state',
      title: this.$t('settings.system.systemSettings.lastLoadedPlaybackState'),
      enabled: false
    }
  }

  get enableJukeboxMode() {
    return {
      key: 'jukebox_mode_toggle',
      title: this.$t('settings.system.systemSettings.enableJukeboxMode'),
      enabled: false
    }
  }

  get jukeboxOptionalFields(): Checkbox[] {
    return [
      {
        key: 'jukebox_skip',
        title: this.$t('settings.system.jukebox.optional_skip'),
        enabled: false
      },
      {
        key: 'jukebox_shuffle',
        title: this.$t('settings.system.jukebox.optional_shuffle'),
        enabled: false
      },
      {
        key: 'jukebox_repeat',
        title: this.$t('settings.system.jukebox.optional_repeat'),
        enabled: false
      }
    ]
  }

  get youtubeEnvExists() {
    return !!(process.env.YoutubeClientID && process.env.YoutubeClientSecret)
  }

  get lastfmEnvExists() {
    return !!(process.env.LastFmApiKey && process.env.LastFmSecret)
  }

  get startupCheckbox(): SystemSettings {
    return {
      key: 'startOnStartup',
      title: this.$t('settings.system.systemSettings.systemStartup'),
      enabled: false
    }
  }

  get minimizeToTrayCheckbox(): SystemSettings {
    return {
      key: 'minimizeToTray',
      title: this.$t('settings.system.systemSettings.minimizeTray'),
      enabled: true
    }
  }

  get hardwareAcceleration(): SystemSettings {
    return {
      key: 'hardwareAcceleration',
      title: this.$t('settings.system.systemSettings.hardwareAcceleration'),
      enabled: true
    }
  }

  get watchFileChanges(): SystemSettings {
    return {
      key: 'watchFileChanges',
      title: this.$t('settings.system.systemSettings.watchFileChanges'),
      enabled: false
    }
  }

  get queueCheckboxValues(): SystemSettings[] {
    return [
      {
        key: 'clear_queue_playlist',
        title: this.$t('settings.system.queueSettings.clearQueue'),
        enabled: false
      }
    ]
  }

  get volumeMapProviders() {
    const ret = [
      {
        key: 'local',
        title: 'Local',
        clamp: 100
      },
      {
        key: 'youtube',
        title: 'Youtube',
        clamp: 100
      },
      {
        key: 'spotify',
        title: 'Spotify',
        clamp: 100
      }
    ]

    for (const e of this.extensions) {
      ret.push({
        key: e.packageName,
        title: e.name,
        clamp: 100
      })
    }

    return ret
  }

  get scrobbleProviderCheckboxValues(): Checkbox[] {
    const ret = [
      {
        key: 'local',
        title: 'Local',
        enabled: true
      },
      {
        key: 'youtube',
        title: 'Youtube',
        enabled: true
      },
      {
        key: 'spotify',
        title: 'Spotify',
        enabled: true
      }
    ]

    for (const e of this.extensions) {
      ret.push({
        key: e.packageName,
        title: e.name,
        enabled: true
      })
    }

    return ret
  }

  openSpotifyHelp() {
    window.WindowUtils.openExternal('https://moosync.app/wiki/integrations#enabling-spotify-integration')
  }

  openYoutubeHelp() {
    window.WindowUtils.openExternal('https://moosync.app/wiki/integrations#enabling-youtube-integration')
  }

  closeSpotifyAutomateModal() {
    this.$bvModal.hide('spotify-automate-modal')
  }

  onSystemPrefFetch(value: SystemSettings[]) {
    this.defaultSystemSettings = JSON.parse(JSON.stringify(value))
    this.showJukeboxField =
      this.defaultSystemSettings.find((val) => val.key === this.enableJukeboxMode.key)?.enabled ?? false
  }

  onSystemPrefChange(value: SystemSettings[]) {
    if (Array.isArray(value)) {
      for (let i = 0; i < value.length; i++) {
        if (value[i].key === this.hardwareAcceleration.key) {
          if (this.defaultSystemSettings[i]?.enabled !== value[i].enabled) {
            this.showRestartButton = true
            break
          } else {
            this.showRestartButton = false
          }

          if (value[i].key === this.enableJukeboxMode.key) {
            this.showJukeboxField = value[i].enabled
          }
        }
      }
    }
  }

  onYoutubeAlternativesChanged(value: Checkbox[]) {
    if (Array.isArray(value)) {
      this.showYoutubeField = value.find((val) => val.key === 'use_youtube')?.enabled ?? false
      this.showInvidiousField = value.find((val) => val.key === 'use_invidious')?.enabled ?? false
      this.showPipedField = value.find((val) => val.key === 'use_piped')?.enabled ?? false

      for (const val of value) {
        if (val.enabled !== this.defaultYoutubeAlts.find((val1) => val1.key === val.key)?.enabled) {
          this.showRestartButton = true
          break
        } else {
          this.showRestartButton = false
        }
      }
    }
  }

  onYoutubeAlternativesFetched(value: Checkbox[]) {
    this.defaultYoutubeAlts = JSON.parse(JSON.stringify(value))
    this.showYoutubeField = value.find((val) => val.key === 'use_youtube')?.enabled ?? false
    this.showInvidiousField = value.find((val) => val.key === 'use_invidious')?.enabled ?? false
    this.showPipedField = value.find((val) => val.key === 'use_piped')?.enabled ?? false
  }

  async restartApp() {
    await window.WindowUtils.restartApp()
  }

  showSpotifyAutomateDisclaimer() {
    this.$bvModal.show('spotify-automate-modal')
  }

  async onZoomUpdate() {
    await window.WindowUtils.updateZoom()
  }

  refreshPage = false
  showClearPreferencesDisclaimer() {
    this.$bvModal.show('clear-preferences-modal')
  }

  async clearPreferences() {
    await window.PreferenceUtils.resetToDefault()
    await this.restartApp()
  }

  closeClearPreferencesModal() {
    this.$bvModal.hide('clear-preferences-modal')
  }

  async fetchExtensions() {
    this.extensions = await window.ExtensionUtils.getAllExtensions()
  }

  created() {
    this.fetchExtensions()
    this.fetchInvidiousInstances()
    this.fetchPipedInstances()
  }
}
</script>

<style lang="sass" scoped>
.path-selector
  max-width: 750px

.title
  text-align: left

.create-button, .delete-button
  font-size: 16px
  font-weight: 400
  color: var(--textInverse)
  background-color: var(--accent)
  border-radius: 6px
  margin-bottom: 8px
  margin-left: 15px
  padding: 6px 20px 6px 20px
  margin-top: 30px
  border: 0


.delete-button
  background-color: #db2626
  color: white

.close-icon
  position: absolute
  top: 20px
  right: 20px
  width: 14px
  height: 14px

.invidious-details
  color: var(--textSecondary)
  white-space: pre-line
  font-size: 16px
  text-align: left
  width: 100%
  font-weight: 700
</style>
