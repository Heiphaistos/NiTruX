<template>
  <div class="nx-layout nx-layout--compact-sidebar">
    <aside class="nx-nav"><slot name="nav" /></aside>
    <main class="nx-content"><slot /></main>
  </div>
</template>

<style scoped>
.nx-layout--compact-sidebar { display: flex; height: 100vh; max-width: 100vw; }
.nx-nav {
  width: 56px;
  flex-shrink: 0;
  overflow: hidden;
  transition: width 0.15s ease;
  background: var(--nx-bg-overlay);
  border-right: 1px solid var(--nx-border);
}
/* At rest, overflow:hidden also clips vertically, but nothing can scroll
   there anyway (labels are faded out below). Expanded on hover, a long nav
   list must actually be reachable -- overflow:hidden here would silently
   swallow every item past the viewport height with no way to get to it. */
.nx-nav:hover { width: 220px; overflow-y: auto; }
.nx-nav::-webkit-scrollbar { width: 8px; }
.nx-nav::-webkit-scrollbar-track { background: transparent; }
.nx-nav::-webkit-scrollbar-thumb { background: var(--nx-border); border-radius: 4px; }
.nx-nav::-webkit-scrollbar-thumb:hover { background: var(--nx-text-secondary); }
.nx-content { flex: 1; min-width: 0; overflow: auto; padding: 24px; }

/* At rest (56px) there's no room for AppNav's category titles or item
   labels -- they'd just get clipped mid-word by the overflow:hidden
   above. Fade them out until hover expands the sidebar to 220px. */
.nx-nav :deep(.nx-app-nav__title),
.nx-nav :deep(.nx-app-nav__item span) {
  opacity: 0;
  transition: opacity 0.1s ease;
  white-space: nowrap;
}
.nx-nav:hover :deep(.nx-app-nav__title),
.nx-nav:hover :deep(.nx-app-nav__item span) {
  opacity: 1;
}
</style>
