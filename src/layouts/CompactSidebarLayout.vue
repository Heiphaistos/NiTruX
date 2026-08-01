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
