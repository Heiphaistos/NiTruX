<template>
  <div class="nx-layout nx-layout--floating-dock">
    <main class="nx-content"><slot /></main>
    <nav class="nx-dock"><slot name="nav" /></nav>
  </div>
</template>

<style scoped>
.nx-layout--floating-dock { position: relative; height: 100vh; }
.nx-content { height: 100%; overflow: auto; padding: 24px; padding-bottom: 88px; }
.nx-dock { position: absolute; bottom: 20px; left: 50%; transform: translateX(-50%); padding: 10px 16px; border-radius: 16px; background: var(--nx-bg-elevated); border: 1px solid var(--nx-border); box-shadow: 0 8px 24px rgba(0,0,0,0.35); }
</style>
