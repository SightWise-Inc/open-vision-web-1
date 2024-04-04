<script setup lang="ts">
// import HelloWorld from './components/HelloWorld.vue'

import { WebCam } from './js/web-cam.js';
import initializeWasmModule from '/public/wasm/process_canvas.js';

initializeWasmModule()
  .then(() => {
    window.webCam = new WebCam(document.getElementById("app"));
    window.webCam.start();
  })
</script>

<template>
  <div>

  </div>
  <!-- <HelloWorld msg="Vite + Vue" /> -->
</template>

<style scoped>
.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: filter 300ms;
}
.logo:hover {
  filter: drop-shadow(0 0 2em #646cffaa);
}
.logo.vue:hover {
  filter: drop-shadow(0 0 2em #42b883aa);
}
</style>
