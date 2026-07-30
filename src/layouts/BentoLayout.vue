<template>
  <div class="nx-layout nx-layout--bento">
    <header class="nx-nav"><slot name="nav" /></header>
    <main class="nx-content nx-content--bento"><slot /></main>
  </div>
</template>

<style scoped>
.nx-layout--bento { display: flex; flex-direction: column; height: 100vh; }
.nx-nav { padding: 12px 20px; border-bottom: 1px solid var(--nx-border); background: var(--nx-bg-overlay); }
.nx-content--bento { flex: 1; overflow: auto; padding: 20px; display: grid; grid-template-columns: repeat(4, 1fr); grid-auto-rows: 120px; gap: 14px; }
</style>
