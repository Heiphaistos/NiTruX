<template>
  <div class="nx-layout nx-layout--master-detail">
    <aside class="nx-nav"><slot name="nav" /></aside>
    <main class="nx-content"><slot /></main>
  </div>
</template>

<style scoped>
.nx-layout--master-detail { display: flex; height: 100vh; max-width: 100vw; }
.nx-nav {
  width: 280px;
  flex-shrink: 0;
  overflow-y: auto;
  overflow-x: hidden;
  background: var(--nx-bg-elevated);
  border-right: 1px solid var(--nx-border);
}
.nx-nav::-webkit-scrollbar { width: 8px; }
.nx-nav::-webkit-scrollbar-track { background: transparent; }
.nx-nav::-webkit-scrollbar-thumb { background: var(--nx-border); border-radius: 4px; }
.nx-nav::-webkit-scrollbar-thumb:hover { background: var(--nx-text-secondary); }
.nx-content { flex: 1; min-width: 0; overflow: auto; padding: 24px; }

@media (max-width: 700px) {
  .nx-nav { width: 200px; }
}
</style>
