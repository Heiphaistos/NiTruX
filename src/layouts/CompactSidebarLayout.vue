<template>
  <div class="nx-layout nx-layout--compact-sidebar">
    <aside class="nx-nav"><slot name="nav" /></aside>
    <main class="nx-content"><slot /></main>
  </div>
</template>

<style scoped>
.nx-layout--compact-sidebar { display: flex; height: 100vh; }
.nx-nav { width: 56px; flex-shrink: 0; overflow: hidden; transition: width 0.15s ease; background: var(--nx-bg-overlay); border-right: 1px solid var(--nx-border); }
.nx-nav:hover { width: 220px; }
.nx-content { flex: 1; overflow: auto; padding: 24px; }
</style>
