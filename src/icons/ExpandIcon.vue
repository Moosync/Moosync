<!-- 
  ExpandIcon.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <svg
    class="button-grow"
    viewBox="0 0 22 13"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    @click="$emit('click', $event)"
  >
    <path d="M1.00049 12L11.0005 2L21.0005 12" stroke="var(--accent)" stroke-width="2" />
  </svg>
</template>

<style lang="sass" scoped>
*
  cursor: pointer
</style>
