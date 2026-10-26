/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { PORTFOLIO_SCOPE, useCollection } from '../../../../data/react';
import type { RegistryRow } from '../../../../data/types';
import { useProjectName } from '../../projects';
import { sortRegistry } from '../model';
import type { RegistryItem } from '../model';

const useRegistry = (): RegistryItem[] => {
  const { docs } = useCollection<RegistryRow>(PORTFOLIO_SCOPE, 'registry');
  const nameOf = useProjectName();
  return useMemo(
    () => sortRegistry(docs, (row) => `${nameOf(row.project)} ${row.session ?? row.thread}`),
    [docs, nameOf],
  );
};

export { useRegistry };
