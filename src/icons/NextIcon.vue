<!-- 
  NextIcon.vue is a part of Moosync.
  
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
      d="M12.5039 7.3359L3.1094 1.07293C1.78029 0.186861 0 1.13964 0 2.73703V15.263C0 16.8604 1.78029 17.8131 3.1094 16.9271L12.5038 10.6641C13.6913 9.87246 13.6913 8.12754 12.5039 7.3359Z"
      fill="#565656"
    />
  </svg>
</template>

<style lang="sass" scoped>
*
  cursor: pointer
</style>
