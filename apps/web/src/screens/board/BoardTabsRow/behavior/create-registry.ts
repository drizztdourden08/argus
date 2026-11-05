/* @layer renderer-app @kind logic */
import type { Registry, RegistryEntry } from '../BoardTabsRow.type';

const createRegistry = <T>(): Registry<T> => {
  let entries: readonly RegistryEntry<T>[] = [];
  const listeners = new Set<() => void>();
  const commit = (next: readonly RegistryEntry<T>[]) => {
    entries = next;
    listeners.forEach((listener) => listener());
  };
  return {
    set: (key, value) => {
      const known = entries.some((entry) => entry.key === key);
      commit(known ? entries.map((entry) => (entry.key === key ? { key, value } : entry)) : [...entries, { key, value }]);
    },
    clear: (key) => {
      if (entries.some((entry) => entry.key === key)) commit(entries.filter((entry) => entry.key !== key));
    },
    subscribe: (listener) => {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    get: () => entries,
  };
};

export { createRegistry };
