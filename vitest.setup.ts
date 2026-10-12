// Node >= 25 defines its own global localStorage/sessionStorage (undefined
// without --localstorage-file); it shadows jsdom's, so restore jsdom's.
declare const jsdom: { window: Window } | undefined;
if (typeof jsdom !== "undefined") {
  for (const key of ["localStorage", "sessionStorage"] as const) {
    Object.defineProperty(globalThis, key, { value: jsdom.window[key], configurable: true, writable: true });
  }
}
