<!-- 
  GenreIcon.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <svg
    width="20"
    height="20"
    viewBox="0 0 20 20"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    @click="$emit('click', $event)"
  >
    <title>Genre</title>
    <path
      fill-rule="evenodd"
      clip-rule="evenodd"
      d="M19.9733 3.00776C19.9816 2.97251 20.1687 2.13644 19.4204 1.40195C18.672 0.66751 17.8201 0.850958 17.7842 0.859261C17.7275 0.872123 17.6757 0.900412 17.6346 0.940748L15.973 2.57147L15.5945 3.06349L15.1292 3.97678L11.7797 7.1413C11.0936 6.62177 9.73821 5.86755 8.32636 6.60928C7.38527 7.10239 7.1347 7.73076 6.91365 8.28514C6.73752 8.72689 6.5854 9.10835 6.10933 9.45457C5.88791 9.61543 5.47912 9.69541 5.00575 9.78801C3.82317 10.0194 2.20358 10.3362 1.13932 12.267C-0.128571 14.5677 1.75146 16.7977 2.6196 17.6507L2.8659 17.892C3.52581 18.5397 4.99779 19.7627 6.69696 19.7627C7.23154 19.7627 7.78863 19.6416 8.34996 19.3435C10.3173 18.299 10.6402 16.7099 10.8759 15.5495C10.9702 15.085 11.0517 14.6839 11.2156 14.4667C11.5683 13.9993 11.957 13.85 12.4072 13.677C12.9721 13.4599 13.6125 13.2139 14.1145 12.2905C14.8703 10.9047 14.1016 9.5744 13.5722 8.90105L16.796 5.61413L17.7275 5.157L18.2286 4.78534L19.8903 3.15462C19.9314 3.11424 19.9602 3.06336 19.9733 3.00776ZM4.32613 16.1722C5.35326 17.1994 6.91305 17.3049 7.81003 16.4079C8.707 15.5109 8.60148 13.9512 7.57435 12.924C6.54722 11.8969 4.98742 11.7914 4.09045 12.6883C3.19348 13.5853 3.299 15.1451 4.32613 16.1722Z"
      :fill="active ? 'var(--accent)' : 'var(--textPrimary)'"
    />
  </svg>
</template>

<script lang="ts">
import { Vue } from 'vue-facing-decorator'
import { Component, Prop } from 'vue-facing-decorator'

@Component({
  emits: ['click']
})
export default class Genre extends Vue {
  @Prop({ default: false })
  active!: boolean
}
</script>
