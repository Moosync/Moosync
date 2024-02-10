<!-- 
  TabCarousel.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-container fluid>
    <b-row no-gutters>
      <b-col class="song-header-options w-100">
        <b-row no-gutters align-v="center" class="h-100">
          <b-col cols="auto" class="mr-3 h-100 d-flex align-items-center" v-if="items.length > 0 && showPrevIcon">
            <PrevIcon @click="onPrevClick" />
          </b-col>
          <b-col class="provider-outer-container" v-if="items.length > 0">
            <div ref="gradientContainer" class="gradient-overlay" :style="{ background: computedGradient }"></div>
            <div ref="providersContainer" :class="`${alignProvidersToEnd ? 'rtl' : ''} provider-container d-flex`">
              <div
                v-for="provider in sortedItems"
                cols="auto"
                class="`h-100 item-checkbox-col mr-2"
                :key="provider.key"
                @click="onProviderSelected(provider.key)"
              >
                <div
                  class="h-100 d-flex item-checkbox-container"
                  :style="{ background: getItemBackgroundColor(provider), color: getItemTextColor(provider) }"
                >
                  <span class="align-self-center provider-title">{{ provider.title }}</span>
                </div>
              </div>
            </div>
          </b-col>
          <b-col cols="auto" class="ml-3 mr-3 h-100 d-flex align-items-center" v-if="items.length > 0">
            <NextIcon @click="onNextClick" v-if="showNextIcon" />
          </b-col>

          <b-col cols="auto" class="ml-auto d-flex" ref="buttonGroupContainer" v-if="showExtraSongListActions">
            <div v-if="showSearchbar" class="searchbar-container mr-3">
              <b-form-input
                v-model="searchText"
                class="searchbar"
                :placeholder="$t('songView.songList.topbar.searchPlaceholder')"
                type="text"
                ref="searchbar"
                @update="onSearchChange"
              />
            </div>
            <SearchIcon @click="toggleSearch" :accent="false" class="mr-3 align-self-center" />
            <SortIcon v-if="isSortAsc" @click="showSortMenu" class="align-self-center" />
            <SortIconAlt v-else @click="showSortMenu" class="align-self-center" />
          </b-col>
        </b-row>
      </b-col>
    </b-row>
  </b-container>
</template>

<script lang="ts">
import { mixins } from 'vue-facing-decorator'
import { Component, Prop, Ref, Watch } from 'vue-facing-decorator'
import ContextMenuMixin from '@/utils/ui/mixins/ContextMenuMixin'
import SortIcon from '@/icons/SortIcon.vue'
import SortIconAlt from '@/icons/SortIconAlt.vue'

import SearchIcon from '@/icons/SearchIcon.vue'
import NextIcon from '@/icons/NavForwardIcon.vue'
import PrevIcon from '@/icons/NavBackIcon.vue'
import { bus } from '@/mainWindow/main'
import { EventBus } from '@/utils/preload/ipc/constants'

@Component({
  components: {
    SortIcon,
    SortIconAlt,
    SearchIcon,
    NextIcon,
    PrevIcon
  },
  emits: ['onItemsChanged', 'onSortClicked', 'onSearchChange']
})
export default class TabCarousel extends mixins(ContextMenuMixin) {
  @Prop({ default: () => [] })
  items!: TabCarouselItem[]

  @Prop({ default: true })
  showExtraSongListActions!: boolean

  @Prop({ default: false })
  private singleSelectMode!: boolean

  @Prop({ default: true })
  private showBackgroundOnSelect!: boolean

  @Prop({ default: 'var(--secondary)' })
  private defaultBackgroundColor!: string

  @Prop({ default: false })
  alignProvidersToEnd!: boolean

  @Prop({ default: true })
  isSortAsc!: boolean

  @Ref('providersContainer')
  private providerContainer!: HTMLDivElement

  @Ref('gradientContainer')
  private gradientContainer!: HTMLDivElement

  @Ref('buttonGroupContainer')
  private buttonGroupContainer!: HTMLDivElement

  private scrollLeft = 0
  private containerSize = 0

  private selectedProviders: string[] = []

  showSearchbar = false
  searchText = ''

  get showNextIcon() {
    if (this.alignProvidersToEnd) {
      return this.scrollLeft < 0
    }
    return this.scrollLeft + this.containerSize < this.providerContainer?.scrollWidth
  }

  get showPrevIcon() {
    if (this.providerContainer?.scrollWidth > this.containerSize) {
      if (this.alignProvidersToEnd) {
        const scrollDiff = this.containerSize - this.providerContainer?.scrollWidth
        return this.scrollLeft > scrollDiff
      }
      return this.scrollLeft > 0
    }
    return false
  }

  get sortedItems() {
    if (this.alignProvidersToEnd) {
      return this.items.reverse()
    }
    return this.items
  }

  @Watch('items')
  private onItemsChanged(items: TabCarouselItem[]) {
    for (const p of items) {
      if (p.defaultChecked && !this.selectedProviders.find((val) => val === p.key)) {
        this.onProviderSelected(p.key)
      }
    }
  }

  getItemBackgroundColor(provider: TabCarouselItem) {
    if (this.selectedProviders.includes(provider.key)) {
      if (!this.showBackgroundOnSelect) return ''
      return 'var(--textSecondary)'
    } else {
      if (!this.showBackgroundOnSelect) return ''
      return this.defaultBackgroundColor
    }
  }

  getItemTextColor(provider: TabCarouselItem) {
    if (this.selectedProviders.includes(provider.key)) {
      if (!this.showBackgroundOnSelect) return 'var(--textPrimary)'
      return ''
    } else {
      if (!this.showBackgroundOnSelect) return 'var(--textSecondary)'
      return ''
    }
  }

  onProviderSelected(key: string) {
    const isSelected = this.selectedProviders.findIndex((val) => val === key)

    if (!this.singleSelectMode) {
      if (isSelected === -1) {
        this.selectedProviders.push(key)
      } else {
        this.selectedProviders.splice(isSelected, 1)
      }
    } else {
      this.selectedProviders = [key]
    }

    this.$emit('onItemsChanged', { key, checked: isSelected === -1 })
  }

  showSortMenu(event: PointerEvent) {
    event.preventDefault()
    event.stopPropagation()
    this.$emit(
      'onSortClicked',
      new PointerEvent('click', {
        clientX: event.pageX,
        clientY: this.buttonGroupContainer.getBoundingClientRect().top + this.buttonGroupContainer.clientHeight + 10
      })
    )
  }

  toggleSearch() {
    this.showSearchbar = !this.showSearchbar

    if (this.showSearchbar) {
      this.$nextTick().then(() => {
        ;(this.$refs['searchbar'] as HTMLInputElement).focus()
      })
    }
  }

  onSearchChange() {
    this.$emit('onSearchChange', this.searchText)
  }

  private resizeObserver?: ResizeObserver

  mounted() {
    if (this.providerContainer && this.gradientContainer) {
      const scrollProviders = (e: WheelEvent) => {
        e.stopPropagation()
        e.preventDefault()
        if (e.deltaY > 0) this.providerContainer.scrollTo({ left: this.providerContainer.scrollLeft + 20 })
        else this.providerContainer.scrollTo({ left: this.providerContainer.scrollLeft - 20 })
        this.scrollLeft = this.providerContainer.scrollLeft
      }

      this.providerContainer.onwheel = scrollProviders.bind(this)
      this.gradientContainer.onwheel = scrollProviders.bind(this)

      if (this.resizeObserver) {
        this.resizeObserver.disconnect()
      }
      this.resizeObserver = new ResizeObserver((e) => {
        window.requestAnimationFrame(() => {
          this.containerSize = e[0].target.clientWidth
        })
      })
      this.resizeObserver.observe(this.providerContainer)
    }

    this.onItemsChanged(this.items)

    bus.on(EventBus.UPDATE_OPTIONAL_PROVIDER, (providerKey: string) => {
      this.selectedProviders.push(providerKey)
    })
  }

  get computedGradient() {
    return `linear-gradient(90deg, var(--primary) 0% , rgba(255,255,255,0) ${
      this.showPrevIcon ? '10%' : '0%'
    }, rgba(255,255,255,0) ${this.showNextIcon ? '90%' : '100%'}, var(--primary) 100%)`
  }

  onNextClick() {
    const scrollLeft = this.providerContainer.scrollLeft + 100
    this.providerContainer.scrollTo({ left: scrollLeft, behavior: 'smooth' })
    this.scrollLeft = scrollLeft
  }

  onPrevClick() {
    const scrollLeft = this.providerContainer.scrollLeft - 100
    this.providerContainer.scrollTo({ left: scrollLeft, behavior: 'smooth' })
    this.scrollLeft = scrollLeft
  }
}
</script>

<style lang="sass">
.custom-dropdown
  background-color: transparent !important
  border-color: transparent !important
  box-shadow: none !important

.provider-container::-webkit-scrollbar-thumb
  display: none

.provider-container::-webkit-scrollbar
  display: none
</style>

<style lang="sass" scoped>
.song-header-options
  height: 40px
  border-radius: 10px

.item-checkbox-container
  border-radius: 8px
  padding-top: 3px
  padding-bottom: 3px
  padding-left: 15px
  padding-right: 15px
  cursor: pointer

.provider-container
  transition: all 0.3s ease-out
  overflow-x: auto
  min-width: 10%

.gradient-overlay
  position: absolute
  width: 100%
  pointer-events: none
  height: 100%

.provider-outer-container
  min-width: 10%

.searchbar
  color: var(--textPrimary) !important
  background: var(--tertiary)
  border: none
  border-radius: 15px
  height: 100%
  transition: background 0.3s cubic-bezier(0.39, 0.58, 0.57, 1), border-radius 1000ms
  text-align: left
  box-shadow: none
  &::-webkit-input-placeholder
    color: var(--textSecondary)
  &:focus
    background: var(--tertiary) !important
    outline: 0

.provider-title
  font-size: 16px

.rtl
  direction: rtl
</style>
