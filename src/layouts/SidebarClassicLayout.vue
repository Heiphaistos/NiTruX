<template>
  <div class="nx-layout nx-layout--sidebar-classic">
    <aside class="nx-nav"><slot name="nav" /></aside>
    <main class="nx-content"><slot /></main>
  </div>
</template>

<style scoped>
.nx-layout--sidebar-classic { display: flex; height: 100vh; }
.nx-nav { width: 220px; flex-shrink: 0; background: var(--nx-bg-overlay); border-right: 1px solid var(--nx-border); }
.nx-content { flex: 1; overflow: auto; padding: 24px; }
</style>
