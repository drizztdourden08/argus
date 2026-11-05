/* @layer renderer-app @kind logic */
import { useSyncExternalStore } from 'react';
import type { Registry, RegistryEntry } from '../BoardTabsRow.type';

const useRegistry = <T>(registry: Registry<T>): readonly RegistryEntry<T>[] => useSyncExternalStore(registry.subscribe, registry.get);

export { useRegistry };
