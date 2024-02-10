<!-- 
  System.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <div class="w-100 h-100">
    <div class="mb-3">
      <CheckboxGroup key="logs" :title="$t('settings.logs.logSettings')" :defaultValue="logSettings" />
    </div>
    <div class="logger-bg">
      <b-container fluid class="controls w-100 d-flex">
        <b-row class="mt-2 w-100">
          <b-col cols="auto">
            <b-row no-gutters>
              <b-col class="filter-title">{{ $t('settings.logs.level') }}</b-col>
            </b-row>
            <b-row>
              <b-col>
                <b-dropdown :text="capitalizeFirstLetter($t(`settings.logs.levels.${levelFilter.toLowerCase()}`))"
                  variant="success" class="dropdown-container mb-3" toggle-class="inner-dropdown">
                  <b-dropdown-item @click="logLevelChange('ALL')">{{ $t('settings.logs.levels.all') }}</b-dropdown-item>
                  <b-dropdown-item @click="logLevelChange('DEBUG')">{{ $t('settings.logs.levels.debug')
                  }}</b-dropdown-item>
                  <b-dropdown-item @click="logLevelChange('INFO')">{{ $t('settings.logs.levels.info') }}</b-dropdown-item>
                  <b-dropdown-item @click="logLevelChange('WARN')">{{ $t('settings.logs.levels.warn') }}</b-dropdown-item>
                  <b-dropdown-item @click="logLevelChange('ERROR')">{{ $t('settings.logs.levels.error')
                  }}</b-dropdown-item>
                </b-dropdown>
              </b-col>
            </b-row>
          </b-col>
          <b-col cols="auto">
            <b-row no-gutters>
              <b-col class="filter-title">{{ $t('settings.logs.process') }}</b-col>
            </b-row>
            <b-row>
              <b-col>
                <b-dropdown :text="processFilter" variant="success" class="dropdown-container mb-3"
                  toggle-class="inner-dropdown">
                  <b-dropdown-item @click="processFilterChange('All')">All</b-dropdown-item>
                  <b-dropdown-item v-for="process in processFilters" :key="process"
                    @click="processFilterChange(process)">{{ process }}</b-dropdown-item>
                </b-dropdown>
              </b-col>
            </b-row>
          </b-col>
          <b-col>
            <b-row no-gutters>
              <b-col class="filter-title">{{ $t('settings.logs.filterMessage') }}</b-col>
            </b-row>
            <b-row>
              <b-col>
                <b-input-group class="search-group">
                  <template #prepend>
                    <SearchIcon class="align-self-center prepend-icon" />
                  </template>
                  <b-input class="align-self-center search-field" placeholder="Search..." debounce="300"
                    v-model="searchFilter"></b-input>
                </b-input-group>
              </b-col>
            </b-row>
          </b-col>
          <b-col cols="auto" class="mt-3 align-self-center refresh-icon" @click="refreshLogs">
            <RefreshIcon />
          </b-col>
        </b-row>
      </b-container>
      <div class="log-content w-100" no-gutters>
        <b-table :filter-function="handleFilter" hover dark sticky-header sort-by="time" :sort-desc="true"
          :items="logLines" :fields="fields" :filter="filterCriteria" :perPage="perPage" :current-page="currentPage"
          id="logs-table" class="log-table" @filtered="onFiltered">
          <template #cell(time)="data">
            <div :class="`time-col ${data.item.level.toLowerCase()}`">
              {{ data.item.time }}
            </div>
          </template>

          <template #cell(level)="data">
            <div :class="`level-col ${data.item.level.toLowerCase()}`">
              {{ data.item.level }}
            </div>
          </template>

          <template #cell(process)="data">
            <div :class="`process-col ${data.item.level.toLowerCase()}`">
              {{ data.item.process }}
            </div>
          </template>

          <template #cell(message)="data">
            <div class="message-col">
              <pre :class="data.item.level.toLowerCase()">{{ data.item.message }}</pre>
            </div>
          </template>
        </b-table>
      </div>
      <b-pagination v-model="currentPage" :total-rows="totalRows" :per-page="perPage" aria-controls="logs-table"
        class="pagination"></b-pagination>
    </div>
  </div>
</template>

<script lang="ts">
import { Component } from 'vue-facing-decorator'
import { Vue } from 'vue-facing-decorator'
import CheckboxGroup from '../CheckboxGroup.vue'
import SearchIcon from '@/icons/SearchIcon.vue'
import RefreshIcon from '../../../icons/RefreshIcon.vue'
import { BvTableFieldArray } from 'bootstrap-vue'

type LogLevels = 'ALL' | 'DEBUG' | 'INFO' | 'WARN' | 'ERROR'

@Component({
  components: {
    CheckboxGroup,
    SearchIcon,
    RefreshIcon
  }
})
export default class Logs extends Vue {
  logLines: LogLines[] = []

  levelFilter: LogLevels = 'ALL'
  processFilter = 'All'
  possibleProcessFilters: Set<string> = new Set(['Main', 'Renderer', 'Extension Host'])

  searchFilter = ''

  perPage = 50
  currentPage = 1
  totalRows = this.logLines.length

  get logSettings(): Checkbox[] {
    return [
      {
        key: 'debug_logging',
        title: this.$t("settings.logs.debug_logging"),
        enabled: false
      }
    ]
  }

  get processFilters() {
    return Array.from(this.possibleProcessFilters)
  }

  filterCriteria = 'key'

  get fields(): BvTableFieldArray {
    return [
      { key: 'time', sortable: true, 'label': this.$t('settings.logs.table.time') },
      { key: 'level', 'label': this.$t('settings.logs.table.level') },
      { key: 'process', label: this.$t('settings.logs.table.process') },
      { key: 'message', 'label': this.$t('settings.logs.table.message') }
    ]
  }

  handleFilter(val: LogLines) {
    const currentLogLevel = this.getLogLevel(this.levelFilter)
    const itemLogLevel = this.getLogLevel(val.level)

    if (itemLogLevel < currentLogLevel) {
      return false
    }

    if (this.processFilter !== 'All' && val.process !== this.processFilter) {
      return false
    }

    if (this.searchFilter && !val.message.includes(this.searchFilter)) {
      return false
    }

    return true
  }

  getLogLevel(level: LogLevels) {
    switch (level) {
      case 'ERROR':
        return 5
      case 'WARN':
        return 4
      case 'INFO':
        return 3
      case 'DEBUG':
        return 2
      case 'ALL':
        return 1
    }
  }

  processFilterChange(name: string) {
    this.processFilter = name
  }

  logLevelChange(level: LogLevels) {
    this.levelFilter = level
  }

  onFiltered(filteredItems: LogLines[]) {
    this.totalRows = filteredItems.length
  }

  mounted() {
    this.refreshLogs()
  }

  refreshLogs() {
    window.LoggerUtils.watchLogs((data) => {
      this.logLines = data
      const uniqueProcesses = new Set(data.map((item) => item.process))
      this.possibleProcessFilters = new Set([...uniqueProcesses, ...this.possibleProcessFilters])
      window.LoggerUtils.unwatchLogs()
    })
  }

  beforeUnmount() {
    window.LoggerUtils.unwatchLogs()
  }

  capitalizeFirstLetter(str: string) {
    return str.charAt(0).toUpperCase() + str.toLowerCase().slice(1)
  }
}
</script>

<style lang="sass">
.table
  background-color: var(--tertiary) !important
  thead
    tr
      th
        position: sticky !important
        border-top: none
        text-align: left

td
  text-align: left

.bg-b-table-default
  background-color: var(--tertiary) !important

.message-col
  word-break: break-word
  max-width: 600px
  pre
    color: white
    font-family: 'Nunito Sans'
    font-size: 16px
    white-space: pre-wrap

.show > .btn-success.dropdown-toggle
  background-color: var(--secondary) !important

.dropdown-container
  min-width: 100px

.inner-dropdown
  background-color: var(--primary) !important
  border: none !important
  padding: 5px 35px 5px 15px
  border-radius: 13px
  &:focus
    box-shadow: none !important
  &::after
    position: absolute
    right: 5px
    top: 50%
    transform: translate(-50%, -50%)
    margin-left: 10px

.dropdown-menu
  background-color: var(--secondary)

.dropdown-item
  color: var(--textPrimary) !important
  &:hover
    background-color: var(--primary)

.page-link
  background-color: var(--primary) !important
  color: var(--textPrimary) !important
  border-color: var(--secondary)
  &:hover
    border-color: var(--primary)

.page-item.active .page-link
  border-color: var(--textPrimary)

.page-item.disabled .page-link
  border-color: var(--secondary)
</style>

<style lang="sass" scoped>
.logger-bg
  background: var(--tertiary)
  height: calc(100% - 65px - 1rem)
  border-radius: 4px

.log-content
  height: calc( 100% - 160px )

.log-table
  max-height: 100% !important
  color: white !important

.search-field
  background: transparent
  border: none !important
  padding: 5px 35px 5px 15px
  height: 34px
  color: var(--textPrimary)

.search-group
  background: var(--primary)
  border-radius: 13px

.prepend-icon
  margin-left: 15px
  height: 18px

.pagination
  margin-top: 15px
  justify-content: center

.debug
  color: #29B8DB

.info
  color: #2E80EA

.warn
  color: #F5F543

.error
  color: #F04538

.filter-title
  text-align: left
  margin-bottom: 5px
  font-size: 18px

.refresh-icon
  svg
    width: 25px
    height: 25px
</style>
