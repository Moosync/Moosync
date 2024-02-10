<!-- 
  Paths.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <div class="w-100 h-100">
    <b-container fluid>
      <b-row no-gutters class="w-100">
        <div class="path-selector w-100">
          <b-container fluid>
            <b-row no-gutters v-if="scanStatus !== 0">
              <b-col>
                <b-progress class="progress-container mb-4" :max="totalValue || 1">
                  <b-progress-bar class="progress-bar" :value="currentValue || 1" animated />
                </b-progress>
              </b-col>
              <b-col cols="auto" class="ml-3">
                {{ Math.round(Math.min((currentValue || 1 / totalValue || 1) * 100, 100)) }}%
              </b-col>
            </b-row>
          </b-container>

          <DirectoryGroup :title="$t('settings.paths.songDirectories')"
            :tooltip="$t('settings.paths.songDirectories_tooltip')" :defaultValue="[]" key="musicPaths"
            @refresh="forceRescan" :enableCheckbox="false" :showRefreshIcon="true" />

          <DirectoryGroup :title="$t('settings.paths.songDirectories_exclude')"
            :tooltip="$t('settings.paths.songDirectories_exclude_tooltip')" :defaultValue="[]" key="exclude_musicPaths"
            class="mt-2" :enableCheckbox="false" :showRefreshIcon="false" />

          <EditText :title="$t('settings.paths.scan_threads')" :tooltip="$t('settings.paths.scan_threads_tooltip')"
            class="mt-2" key="scan_threads" type="number" :defaultValue="scanThreads" />

          <EditText :title="$t('settings.paths.splitter')" :tooltip="$t('settings.paths.splitter_tooltip')" class="mt-2"
            key="scan_splitter" :defaultValue="splitterRegex" />

          <EditText :title="$t('settings.paths.scan_interval')" :tooltip="$t('settings.paths.scan_interval_tooltip')"
            class="mt-2" type="number" key="scan_interval" :onValueChange="onScanIntervalChange" :defaultValue="30" />

          <FilePicker :title="$t('settings.paths.artworkPath')" :tooltip="$t('settings.paths.artworkPath_tooltip')"
            key="artworkPath" class="mt-5" />
          <FilePicker :title="$t('settings.paths.thumbnailPath')" :tooltip="$t('settings.paths.thumbnailPath_tooltip')"
            key="thumbnailPath" class="mt-5" />
        </div>
      </b-row>
    </b-container>
  </div>
</template>

<script lang="ts">
import { Component } from 'vue-facing-decorator'
import { Vue } from 'vue-facing-decorator'
import FilePicker from '../FilePicker.vue'
import DirectoryGroup from '../DirectoryGroup.vue'
import EditText from '../EditText.vue'
import { ScanStatus } from '@/utils/commonConstants'

@Component({
  components: {
    DirectoryGroup,
    FilePicker,
    EditText
  }
})
export default class Paths extends Vue {
  currentValue = 0
  totalValue = 0
  scanStatus: ScanStatus = ScanStatus.UNDEFINED

  cpuCount = 0

  forceRescan() {
    window.FileUtils.scan(true)
  }

  setProgress(progress: Progress) {
    this.currentValue = progress.current
    this.totalValue = progress.total
    this.scanStatus = progress.status
  }

  private openWiki() {
    window.WindowUtils.openExternal('https://moosync.app/wiki/#known-bugs')
  }

  get splitterRegex() {
    return ';'
  }

  get scanThreads() {
    return this.cpuCount
  }

  async created() {
    this.cpuCount = await window.FileUtils.getCPUs()
    this.setProgress(await window.FileUtils.getScanProgress())
    window.FileUtils.listenScanProgress(async (progress) => {
      this.setProgress(progress)
    })
  }

  async onScanIntervalChange() {
    await window.FileUtils.resetScanTask()
  }
}
</script>

<style lang="sass" scoped>
.path-selector
  max-width: 750px

.title
  text-align: left

.progress-bar
  background-color: var(--accent)

.progress-container
  font-size: 16px
  height: 1.3rem !important
  background-color: var(--tertiary)

.lib-missing
  text-align: left
  margin-bottom: 15px
  color: #E62017

.lib-missing-link
  cursor: pointer
  &:hover
    text-decoration: underline
</style>
