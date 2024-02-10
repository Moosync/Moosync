<!-- 
  PlaylistDefaultIcon.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <svg viewBox="0 0 201 201" fill="none" xmlns="http://www.w3.org/2000/svg" @click="$emit('click', $event)">
    <rect width="201" height="201" fill="var(--secondary)" />
    <path
      d="M107.416 129.83C107.416 142.24 117.51 152.333 129.92 152.333C142.329 152.333 152.423 142.24 152.423 129.83C152.423 128.694 152.25 127.603 152.083 126.513H152.333V62.4998H165.166V49.6665H145.916C144.215 49.6665 142.582 50.3425 141.379 51.5459C140.176 52.7493 139.5 54.3814 139.5 56.0832V109.553C136.512 108.107 133.239 107.349 129.92 107.333C123.954 107.338 118.234 109.71 114.015 113.928C109.796 118.145 107.423 123.864 107.416 129.83V129.83ZM36.833 56.0832H126.666V68.9165H36.833V56.0832Z"
      fill="var(--textSecondary)"
    />
    <path
      d="M36.833 81.75H126.666V94.5833H36.833V81.75ZM36.833 107.417H94.583V120.25H36.833V107.417ZM36.833 133.083H94.583V145.917H36.833V133.083Z"
      fill="var(--textSecondary)"
    />
  </svg>
</template>
