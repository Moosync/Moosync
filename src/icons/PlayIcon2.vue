<!-- 
  PlayIcon2.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <svg
    width="23"
    height="26"
    viewBox="0 0 23 26"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    @click="$emit('click', $event)"
  >
    <path
      d="M21.789 10.9007L3.71707 0.333515C2.24872 -0.524653 0 0.308125 0 2.4307V23.5599C0 25.4641 2.08957 26.6117 3.71707 25.6571L21.789 15.095C23.4011 14.1556 23.4062 11.8401 21.789 10.9007V10.9007Z"
      fill="white"
    />
  </svg>
</template>

<style lang="sass" scoped>
*
  cursor: pointer
</style>
