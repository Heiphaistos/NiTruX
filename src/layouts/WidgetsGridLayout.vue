<template>
  <div class="nx-layout nx-layout--widgets-grid">
    <header class="nx-nav"><slot name="nav" /></header>
    <main class="nx-content nx-content--grid"><slot /></main>
  </div>
</template>

<style scoped>
.nx-layout--widgets-grid { display: flex; flex-direction: column; height: 100vh; }
.nx-nav { padding: 12px 20px; border-bottom: 1px solid var(--nx-border); background: var(--nx-bg-overlay); }
.nx-content--grid { flex: 1; overflow: auto; padding: 20px; display: grid; grid-template-columns: repeat(auto-fill, minmax(240px, 1fr)); gap: 16px; align-content: start; }
</style>
