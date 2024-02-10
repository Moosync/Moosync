import { BvModal } from 'bootstrap-vue'

declare module '@vue/runtime-core' {
  interface ComponentCustomProperties {
    $bvModal: BvModal
  }
}
