<template>
  <div class="nx-layout nx-layout--bento">
    <header class="nx-nav"><slot name="nav" /></header>
    <main class="nx-content"><slot /></main>
  </div>
</template>

<style scoped>
.nx-layout--bento { display: flex; flex-direction: column; height: 100vh; }
.nx-nav { padding: 8px 20px; min-height: 52px; display: flex; align-items: center; border-bottom: 1px solid var(--nx-border); background: var(--nx-bg-overlay); }
.nx-content { flex: 1; overflow: auto; padding: 20px; }
</style>
