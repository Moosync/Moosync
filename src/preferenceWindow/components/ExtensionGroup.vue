<!-- 
  ExtensionGroup.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-container fluid class="path-container w-100">
    <b-row no-gutters>
      <PreferenceHeader
        :title="$t('settings.extensions.extensions')"
        tooltip="$t('settings.extensions.extensions_tooltip')"
      />
      <b-col cols="auto" align-self="center" class="new-directories ml-auto d-flex">
        <div class="d-flex" @click="openDiscoverModal">
          <DiscoverIcon class="discover-icon mr-2" />
          <div class="discover-button mr-4">{{ $t('settings.extensions.discover') }}</div>
        </div>

        <div class="d-flex" @click="openFileBrowser">
          <InstallIcon class="install-icon mr-2" />
          <div class="add-directories-button">{{ $t('settings.extensions.install') }}</div>
        </div>
      </b-col>
    </b-row>
    <b-row no-gutters class="background w-100 mt-2 d-flex" v-if="Array.isArray(extensions)">
      <b-row no-gutters class="mt-3 item w-100" v-for="(ext, index) in extensions" :key="ext.packageName">
        <b-col cols="auto" align-self="center" class="ml-4">
          <b-checkbox @change="togglePath(index)" :id="`ext-${index}`" :checked="ext.hasStarted" />
        </b-col>
        <b-col col md="8" lg="9" align-self="center" class="ml-3 justify-content-start">
          <div class="item-text text-truncate">{{ ext.name }} - {{ ext.version }}</div>
          <div class="item-text-subtitle text-truncate">{{ ext.author }}</div>
        </b-col>
        <b-col cols="auto" align-self="center" class="ml-auto">
          <div class="remove-button w-100" @click="removePath(index)">{{ $t('settings.extensions.remove') }}</div>
        </b-col>
      </b-row>
    </b-row>
    <DeleteModal
      v-if="extensions[extensionInAction]"
      id="extensionDeleteModal"
      :itemName="extensions[extensionInAction].name"
      @confirm="removeExtension"
    />
    <DiscoverExtensionsModal
      :updateExtensionsCallback="emitExtensionsUpdated"
      :installedExtensions="extensions"
      id="discoverExtensions"
    />
  </b-container>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-facing-decorator'
import DeleteModal from '../../commonComponents/ConfirmationModal.vue'
import PreferenceHeader from './PreferenceHeader.vue'
import DiscoverExtensionsModal from './DiscoverExtensionModal.vue'
import InstallIcon from '@/icons/InstallIcon.vue'
import DiscoverIcon from '@/icons/DiscoverIcon.vue'
@Component({
  components: {
    DeleteModal,
    DiscoverExtensionsModal,
    PreferenceHeader,
    DiscoverIcon,
    InstallIcon
  },
  emits: ['extensionsChanged']
})
export default class ExtensionGroup extends Vue {
  @Prop({ default: () => [] })
  extensions!: ExtensionDetails[]

  extensionInAction = 0

  togglePath(index: number) {
    if (index >= 0) {
      const ext = this.extensions[index]
      const status = (document.getElementById(`ext-${index}`) as HTMLInputElement).checked
      window.ExtensionUtils.toggleExtStatus(ext.packageName, status).then(() => (ext.hasStarted = status))
    }
  }

  removeExtension() {
    if (this.extensions[this.extensionInAction]) {
      window.ExtensionUtils.uninstall(this.extensions[this.extensionInAction].packageName).then(() =>
        this.emitExtensionsUpdated()
      )
    }
  }

  removePath(index: number) {
    if (index >= 0) {
      this.extensionInAction = index
      this.$bvModal.show('extensionDeleteModal')
      // window.ExtensionUtils.uninstall(this.extensions[index].packageName).then(() => this.$emit('extensionsChanged'))
    }
  }

  openFileBrowser() {
    window.WindowUtils.openFileBrowser(false, true, [{ name: 'Moosync Extension File', extensions: ['msox'] }]).then(
      (data) => {
        if (!data.canceled) {
          window.ExtensionUtils.install(...data.filePaths).then(() => this.emitExtensionsUpdated())
        }
      }
    )
  }

  emitExtensionsUpdated() {
    this.$emit('extensionsChanged')
  }

  openDiscoverModal() {
    this.$bvModal.show('discoverExtensions')
  }
}
</script>

<style lang="sass">
.custom-control-input:checked + .custom-control-label::before
  background-color: transparent
  border-color: var(--textPrimary)

.custom-control-input:indeterminate ~ .custom-control-label
  background-image: none
  box-shadow: none

.custom-control-input:focus ~ .custom-control-label::before
  outline: var(--textPrimary) !important
  border: 1px solid var(--textPrimary) !important
  box-shadow: 0 0 1px 1px var(--textPrimary)

.custom-control-label
  &::before
    background-color: transparent
</style>

<style lang="sass" scoped>
.title
  font-size: 20px

.new-directories
  font-size: 18px
  color: var(--accent)
  &:hover
    cursor: pointer

.add-directories-button
  user-select: none

.background
  align-content: flex-start
  background-color: var(--tertiary)
  height: 220px
  overflow-y: scroll
  overflow-x: hidden

  &::-webkit-scrollbar-track
    background: var(--tertiary)

.item
  height: 35px
  flex-wrap: nowrap

.item-text
  font-size: 18px
  color: var(--textPrimary)
  min-width: 0
  text-align: left

.item-text-subtitle
  font-size: 14px
  color: var(--textSecondary)
  min-width: 0
  text-align: left


.remove-button
  color: #E62017
  user-select: none
  &:hover
    cursor: pointer

.discover-icon, .install-icon
  height: 24px
</style>
