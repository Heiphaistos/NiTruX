<template>
  <div class="nx-layout nx-layout--command-first">
    <div class="nx-command-bar"><slot name="nav" /></div>
    <main class="nx-content"><slot /></main>
  </div>
</template>

<style scoped>
.nx-layout--command-first { display: flex; flex-direction: column; height: 100vh; align-items: center; }
.nx-command-bar { width: min(640px, 90%); margin-top: 24px; padding: 10px 16px; border-radius: 12px; background: var(--nx-bg-elevated); border: 1px solid var(--nx-border); }
.nx-content { flex: 1; width: 100%; overflow: auto; padding: 24px; }
</style>
