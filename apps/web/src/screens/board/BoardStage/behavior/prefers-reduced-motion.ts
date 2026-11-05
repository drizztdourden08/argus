/* @layer renderer-app @kind logic */
const prefersReducedMotion = (): boolean => typeof window.matchMedia === 'function' && window.matchMedia('(prefers-reduced-motion: reduce)').matches;

export { prefersReducedMotion };
