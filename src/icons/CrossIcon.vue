<template>
  <svg viewBox="0 0 8 8" fill="none" xmlns="http://www.w3.org/2000/svg" @click="$emit('click', $event)">
    <path d="M7.25 0.75L0.75 7.25M0.75 0.75L7.25 7.25" :stroke="color" stroke-width="1.5" stroke-linecap="round"
      stroke-linejoin="round" />
  </svg>
</template>

<script lang="ts">
import { Vue } from 'vue-facing-decorator'
import { Component, Prop } from 'vue-facing-decorator'

@Component({
  emits: ['click']
})
export default class CrossIcon extends Vue {
  @Prop({ default: 'var(--textPrimary)' })
  color!: string
}
</script>


<style lang="sass" scoped>
svg
  cursor: pointer
</style>