<template>
  <b-modal
    :id="id"
    class="playlistCreateTypeModal"
    hide-footer
    hide-header
    body-class="multi-modal-body d-flex"
    content-class="multi-modal-content"
    dialog-class="multi-modal-dialog"
    centered
  >
    <b-container>
      <b-row class="h-100">
        <b-col v-for="i of slots" :key="i" class="d-flex" @click="emit(`click-${i}`)">
          <b-row class="item-box align-self-center">
            <b-col cols="auto" class="d-flex item-container">
              <b-row>
                <b-col class="d-flex justify-content-center">
                  <div class="item-icon">
                    <slot class="item-icon" :name="i" />
                  </div>
                </b-col>
              </b-row>
              <b-row>
                <b-col class="d-flex justify-content-center item-title"> <slot :name="`${i}-title`" /> </b-col>
              </b-row>
            </b-col>
          </b-row>
        </b-col>
      </b-row>
    </b-container>
  </b-modal>
</template>

<script lang="ts">
import { Vue } from 'vue-facing-decorator'
import { Component, Prop, Watch } from 'vue-facing-decorator'

@Component({})
export default class MultiButtonModal extends Vue {
  @Prop({ default: 'multiButtonModal' })
  id!: string

  @Prop({ default: false })
  private show!: boolean

  @Prop({ default: 0 })
  slots!: number

  @Watch('show')
  async onShowChange() {
    this.$bvModal.show(this.id)
  }

  emit(e: string) {
    this.$bvModal.hide(this.id)
    this.$emit(e)
  }
}
</script>

<style lang="sass">
.item-box
  height: 200px
  width: 200px
  background: var(--secondary)
  border-radius: 32px
  margin: 10px
  cursor: pointer

  &:hover
    .item-title
      text-decoration: underline


.item-icon
  width: 50%
  svg
    width: 100%

.item-container
  flex-direction: column
  justify-content: center
  align-items: center
  row-gap: 15px

.item-title
  font-size: 18px
  font-weight: 700

.multi-modal-body
  height: 300px

.multi-modal-content
  width: 534px

.multi-modal-dialog
  max-width: 534px
</style>
