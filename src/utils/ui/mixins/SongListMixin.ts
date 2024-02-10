/*
 *  SongListMixin.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { Component, Prop, Vue } from 'vue-facing-decorator'

@Component({
  emits: ['onRowSelectionClear', 'onRowSelected'],
})
export default class SongListMixin extends Vue {
  private lastSelect = ''
  selected: number[] = []

  private keyPressed: 'Control' | 'Shift' | undefined

  @Prop({ default: () => [] })
  songList!: Song[]

  // Clear selection after table loses focus
  clearSelection() {
    this.$emit('onRowSelectionClear')
    this.selected = []
  }

  selectAll() {
    this.selected = Array.from({ length: this.songList.length }, (_, i) => i)
  }

  private onKeyUp(e: KeyboardEvent) {
    if (e.key === 'Shift' && this.keyPressed === 'Shift') this.keyPressed = undefined
    else if (e.key === 'Control' && this.keyPressed === 'Control') this.keyPressed = undefined
  }

  private onKeyDown(e: KeyboardEvent) {
    if ((e.target as HTMLElement)?.tagName?.toLocaleLowerCase() === 'input') return
    if (e.shiftKey || e.ctrlKey) this.keyPressed = e.key as 'Shift' | 'Control'
    if (e.ctrlKey && e.key === 'a') this.selectAll()
  }

  private setupKeyEvents() {
    document.addEventListener('keydown', this.onKeyDown)
    document.addEventListener('keyup', this.onKeyUp)
  }

  private destroyKeyEvents() {
    document.removeEventListener('keydown', this.onKeyDown)
    document.removeEventListener('keyup', this.onKeyUp)
  }

  onRowSelected(index: number) {
    if (this.keyPressed === 'Control') {
      const i = this.selected.findIndex((val) => val === index)
      if (i === -1) {
        this.selected.push(index)
      } else {
        this.selected.splice(i, 1)
      }
    } else if (this.keyPressed === 'Shift') {
      if (this.selected.length > 0) {
        const lastSelected = this.selected[0]
        const min = Math.min(lastSelected, index)
        const max = Math.max(lastSelected, index)
        this.selected = Array.from({ length: max - min + 1 }, (_, i) => min + i)
      }
    } else this.selected = [index]
    this.$emit(
      'onRowSelected',
      this.selected.map((val) => this.songList[val]),
    )
  }

  mounted() {
    this.setupKeyEvents()
  }

  beforeUnmount() {
    this.destroyKeyEvents()
  }
}
