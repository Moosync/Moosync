<!-- 
  ArtistDefaultIcon.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <svg viewBox="0 0 201 201" fill="none" xmlns="http://www.w3.org/2000/svg" @click="$emit('click', $event)">
    <rect width="201" height="201" fill="var(--secondary)" />
    <path
      d="M51.875 158.75C51.875 158.75 42.25 158.75 42.25 149.125C42.25 139.5 51.875 110.625 100 110.625C148.125 110.625 157.75 139.5 157.75 149.125C157.75 158.75 148.125 158.75 148.125 158.75H51.875ZM100 101C107.658 101 115.003 97.9578 120.418 92.5427C125.833 87.1276 128.875 79.7831 128.875 72.125C128.875 64.4669 125.833 57.1224 120.418 51.7073C115.003 46.2922 107.658 43.25 100 43.25C92.3419 43.25 84.9974 46.2922 79.5823 51.7073C74.1672 57.1224 71.125 64.4669 71.125 72.125C71.125 79.7831 74.1672 87.1276 79.5823 92.5427C84.9974 97.9578 92.3419 101 100 101V101Z"
      fill="var(--textSecondary)"
    />
  </svg>
</template>
