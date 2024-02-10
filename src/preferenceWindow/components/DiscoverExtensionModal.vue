<!-- 
  DiscoverExtensionModal.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-modal centered size="xl" :id="id" :ref="id" hide-footer hide-header>
    <div class="modal-content-container">
      <div class="heading">Discover new extensions</div>
      <b-container fluid class="overflow-container">
        <b-row
          align-v="center"
          v-for="(ext, index) of fetchedExtensions"
          :key="index"
          no-gutters
          :class="`${index !== 0 ? 'mt-4' : ''} d-flex`"
        >
          <b-col cols="auto">
            <div class="img-container">
              <img :src="ext.logo" :alt="ext.name + ' logo'" referrerPolicy="no-referrer" />
            </div>
          </b-col>
          <b-col class="ml-3 text-truncate">
            <b-row>
              <b-col class="text-truncate title" :title="ext.name" @click="titleClick(ext)">
                {{ ext.name }}
              </b-col>
            </b-row>
            <b-row>
              <b-col :title="ext.description" class="subtitle text-truncate">{{ ext.description }}</b-col>
            </b-row>
          </b-col>
          <b-col cols="1" class="text-center ml-3">
            <div>{{ ext.release.version }}</div>
          </b-col>
          <b-col
            v-if="ext.progress"
            :class="`ml-3 text-center ${ext.progress.error ? 'error-status' : 'download-status'}`"
            cols="2"
            >{{ ext.progress.error ? ext.progress.error : ext.progress.status }}...</b-col
          >
          <b-col v-else class="ml-3 text-center download-button" cols="2" @click="downloadExt(ext)">{{
            getDownloadButtonText(ext)
          }}</b-col>
        </b-row>
      </b-container>
    </div>
  </b-modal>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-facing-decorator'
import yaml from 'js-yaml'
import semver from 'semver'
import { convertProxy } from '../../utils/ui/common'

@Component({
  components: {}
})
export default class DiscoverExtensionsModal extends Vue {
  @Prop({ default: 'discoverExtensions' })
  id!: string

  @Prop({ default: () => [] })
  private installedExtensions!: ExtensionDetails[]

  // eslint-disable-next-line @typescript-eslint/no-empty-function
  @Prop()
  private updateExtensionsCallback!: (() => void) | undefined

  fetchedExtensions: (FetchedExtensionManifest & { progress?: ExtInstallStatus })[] = []

  private close() {
    this.$bvModal.hide(this.id)
  }

  getDownloadButtonText(ext: FetchedExtensionManifest) {
    const installedExt = this.installedExtensions.find((val) => val.packageName === ext.packageName)
    if (installedExt) {
      if (semver.gt(ext.release.version, installedExt.version)) {
        return 'Update'
      } else {
        return ''
      }
    } else {
      return 'Download'
    }
  }

  private async fetchData() {
    const resp = await fetch('https://api.github.com/repos/Moosync/moosync-exts/git/trees/main?recursive=1')

    const parsedResp: GithubRepoResponse = await resp.json()

    const extensions: string[] = []

    for (const r of parsedResp.tree) {
      if (r.type === 'blob' && r.path.endsWith('/extension.yml')) {
        extensions.push(r.path)
      }
    }

    for (const e of extensions) {
      const manifest = await (
        await fetch(`https://raw.githubusercontent.com/Moosync/moosync-exts/main/${encodeURI(e)}`)
      ).text()
      const parsed = yaml.load(manifest) as FetchedExtensionManifest

      if (parsed) {
        if (!parsed.logo.startsWith('http')) {
          parsed.logo =
            'https://raw.githubusercontent.com/Moosync/moosync-exts/main/' + e.slice(0, -14) + '/' + parsed.logo
        }
        this.fetchedExtensions.push(parsed)
      }
    }
  }

  async downloadExt(ext: FetchedExtensionManifest) {
    const status = await window.ExtensionUtils.downloadExtension(convertProxy(ext))
    console.debug('Extension download and install status', status)
    this.updateExtensionsCallback && this.updateExtensionsCallback()
  }

  private listenInstallStatus() {
    window.ExtensionUtils.listenExtInstallStatus((data) => {
      const extIndex = this.fetchedExtensions.findIndex((val) => val.packageName === data.packageName)
      if (extIndex !== -1) {
        const ext = this.fetchedExtensions[extIndex]
        this.fetchedExtensions.splice(extIndex, 1, {
          ...ext,
          progress: data
        })
      }
    })
  }

  titleClick(ext: FetchedExtensionManifest) {
    window.WindowUtils.openExternal(ext.url)
  }

  mounted() {
    this.listenInstallStatus()
    this.fetchData()
  }
}
</script>

<style lang="sass" scoped>
.modal-content-container
  max-height: 500px
  margin: 20px 15px 20px 20px !important

.heading
  font-size: 26px
  font-weight: 700
  text-align: center
  margin-bottom: 10px

.overflow-container
  overflow-y: auto
  overflow-x: hidden
  max-height: 450px

.img-container
  img
    width: 50px
    border-radius: 0px !important

.title
  cursor: pointer
  font-size: 20px
  &:hover
    text-decoration: underline
.subtitle
  color: var(--textSecondary)

.download-button,
.download-status,
.error-status
  font-size: 18px
  color: var(--accent)

.error-status
  color: red !important

.download-button
  cursor: pointer
  &:hover
    text-decoration: underline
</style>
