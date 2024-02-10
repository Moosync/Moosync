<!-- 
  AnimatedEqualizerIcon.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <svg
    id="eF20KXoiB5d1"
    xmlns="http://www.w3.org/2000/svg"
    xmlns:xlink="http://www.w3.org/1999/xlink"
    viewBox="0 0 12 14"
    shape-rendering="geometricPrecision"
    text-rendering="geometricPrecision"
    @click="$emit('click', $event)"
  >
    <g
      id="eF20KXoiB5d2_ts"
      :class="isRunning ? 'animation-play' : 'animation-pause'"
      transform="translate(1.989899,14) scale(1,1)"
    >
      <path
        id="eF20KXoiB5d2"
        d="M3.337220,0.381592L0.642578,0.381592L0.642578,13.991600L3.337220,13.991600L3.337220,0.381592Z"
        transform="translate(-1.989899,-13.991600)"
        fill="#ffffff"
        stroke="none"
        stroke-width="1"
        stroke-miterlimit="1"
      />
    </g>
    <g
      id="eF20KXoiB5d3_ts"
      :class="isRunning ? 'animation-play' : 'animation-pause'"
      transform="translate(6.030915,14.000671) scale(1,1)"
    >
      <path
        id="eF20KXoiB5d3"
        d="M7.378240,5.199710L4.683590,5.199710L4.683590,13.991700L7.378240,13.991700L7.378240,5.199710Z"
        transform="translate(-6.030915,-13.991600)"
        fill="#ffffff"
        stroke="none"
        stroke-width="1"
        stroke-miterlimit="1"
      />
    </g>
    <g
      id="eF20KXoiB5d4_ts"
      :class="isRunning ? 'animation-play' : 'animation-pause'"
      transform="translate(9.851647,14.039325) scale(1,1)"
    >
      <path
        id="eF20KXoiB5d4"
        d="M11.175100,8.803340L8.480470,8.803340L8.480470,13.991600L11.175100,13.991600L11.175100,8.803340Z"
        transform="translate(-9.827785,-13.991600)"
        fill="#ffffff"
        stroke="none"
        stroke-width="1"
        stroke-miterlimit="1"
      />
    </g>
  </svg>
</template>

<script lang="ts">
import { Vue } from 'vue-facing-decorator'
import { Component, Prop } from 'vue-facing-decorator'

@Component({
  emits: ['click']
})
export default class AnimatedEqualizerIcon extends Vue {
  @Prop({ default: false })
  isRunning!: boolean
}
</script>

<style>
.animation-play {
  animation-play-state: running !important;
}

.animation-pause {
  animation-play-state: paused !important;
}

#eF20KXoiB5d2_ts {
  animation: eF20KXoiB5d2_ts__ts 800ms linear infinite normal forwards;
}
@keyframes eF20KXoiB5d2_ts__ts {
  0% {
    transform: translate(1.989899px, 14px) scale(1, 1);
  }
  25% {
    transform: translate(1.989899px, 14px) scale(1, 0.645321);
  }
  50% {
    transform: translate(1.989899px, 14px) scale(1, 0.380465);
  }
  75% {
    transform: translate(1.989899px, 14px) scale(1, 0.645169);
  }
  100% {
    transform: translate(1.989899px, 14px) scale(1, 0.999765);
  }
}
#eF20KXoiB5d3_ts {
  animation: eF20KXoiB5d3_ts__ts 800ms linear infinite normal forwards;
}
@keyframes eF20KXoiB5d3_ts__ts {
  0% {
    transform: translate(6.030915px, 14.000671px) scale(1, 1);
  }
  25% {
    transform: translate(6.030915px, 14.000671px) scale(1, 0.588887);
  }
  50% {
    transform: translate(6.030915px, 14.000671px) scale(1, 0.999748);
  }
  75% {
    transform: translate(6.030915px, 14.000671px) scale(1, 1.547215);
  }
  100% {
    transform: translate(6.030915px, 14.000671px) scale(1, 0.999435);
  }
}
#eF20KXoiB5d4_ts {
  animation: eF20KXoiB5d4_ts__ts 800ms linear infinite normal forwards;
}
@keyframes eF20KXoiB5d4_ts__ts {
  0% {
    transform: translate(9.851647px, 14.039325px) scale(1, 1);
  }
  25% {
    transform: translate(9.851647px, 14.039325px) scale(1, 1.691557);
  }
  50% {
    transform: translate(9.851647px, 14.039325px) scale(1, 2.623789);
  }
  75% {
    transform: translate(9.851647px, 14.039325px) scale(1, 1.694809);
  }
  100% {
    transform: translate(9.851647px, 14.039325px) scale(1, 0.997334);
  }
}
</style>
