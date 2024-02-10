<!-- 
  LogoIcon.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <svg
    width="23"
    height="23"
    viewBox="0 0 23 23"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    @click="$emit('click', $event)"
  >
    <g clip-path="url(#clip0)">
      <path
        d="M5.37647 16.6102L7.46805 6.12376C7.62136 5.35508 8.36879 4.85623 9.13747 5.00955C9.90614 5.16287 10.405 5.91029 10.2517 6.67897L9.52673 10.3136C9.29937 11.4535 10.0391 12.5619 11.1791 12.7893C12.319 13.0166 13.4274 12.2769 13.6547 11.1369L14.3799 7.50116C14.5331 6.73312 15.2799 6.23469 16.0479 6.38788C16.8157 6.54101 17.3141 7.28734 17.1614 8.05517L15.0608 18.6156C14.8292 19.7801 15.5851 20.9119 16.7494 21.1442C17.9142 21.3765 19.0468 20.6206 19.2791 19.4558L21.6062 7.78882C22.0707 5.45975 20.5592 3.19509 18.2301 2.73054L8.67128 0.82398C6.34221 0.359437 4.07754 1.87093 3.613 4.2L1.29989 15.7972C1.07536 16.9229 1.80592 18.0175 2.93163 18.242C4.05735 18.4665 5.15194 17.736 5.37647 16.6102Z"
        stroke="var(--accent)"
        stroke-width="0.573362"
      />
    </g>
    <defs>
      <clipPath id="clip0">
        <rect width="22.11" height="22.0171" fill="white" />
      </clipPath>
    </defs>
  </svg>
</template>
