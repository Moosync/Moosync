<!-- 
  PrevIcon.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <svg
    width="14"
    height="18"
    viewBox="0 0 14 18"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    @click="$emit('click', $event)"
  >
    <path
      d="M1.49615 7.3359L10.8906 1.07293C12.2197 0.186861 14 1.13964 14 2.73703V15.263C14 16.8604 12.2197 17.8131 10.8906 16.9271L1.49615 10.6641C0.308685 9.87246 0.308683 8.12754 1.49615 7.3359Z"
      fill="#565656"
    />
  </svg>
</template>

<style lang="sass" scoped>
*
  cursor: pointer
</style>
