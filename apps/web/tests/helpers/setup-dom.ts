/* @layer renderer-app @kind test */
const noop = (): void => undefined;

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: (query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addEventListener: noop,
    removeEventListener: noop,
    addListener: noop,
    removeListener: noop,
    dispatchEvent: () => false,
  }),
});

Element.prototype.scrollIntoView = noop;

Object.defineProperty(window, 'CSS', { writable: true, value: { supports: () => false } });
