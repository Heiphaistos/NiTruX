<template>
  <div class="nx-layout nx-layout--master-detail">
    <aside class="nx-nav"><slot name="nav" /></aside>
    <main class="nx-content"><slot /></main>
  </div>
</template>

<style scoped>
.nx-layout--master-detail { display: flex; height: 100vh; }
.nx-nav { width: 280px; flex-shrink: 0; overflow: auto; background: var(--nx-bg-elevated); border-right: 1px solid var(--nx-border); }
.nx-content { flex: 1; overflow: auto; padding: 24px; }
</style>
