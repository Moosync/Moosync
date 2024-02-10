<!-- 
  index.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<script lang="ts">
import { Component, Vue } from 'vue-facing-decorator'

@Component({})
export default class Sidebar extends Vue {
  public render() {
    return null
  }
  created() {
    this.$router.replace('/songs/')
  }
}
</script>
