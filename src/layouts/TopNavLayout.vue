<template>
  <div class="nx-layout nx-layout--top-nav">
    <header class="nx-nav"><slot name="nav" /></header>
    <main class="nx-content"><slot /></main>
  </div>
</template>

<style scoped>
.nx-layout--top-nav { display: flex; flex-direction: column; height: 100vh; }
.nx-nav { padding: 0 20px; height: 52px; display: flex; align-items: center; background: var(--nx-bg-overlay); border-bottom: 1px solid var(--nx-border); }
.nx-content { flex: 1; overflow: auto; padding: 24px; }
</style>
