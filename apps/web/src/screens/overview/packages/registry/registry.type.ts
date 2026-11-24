/* @layer renderer-app @kind types */
import type { RegistryBuild, RegistryConsumer } from '../../../../data/remote';
import type { BuildRow } from './model';

interface BuildsTableProps {
  builds: readonly RegistryBuild[];
}

interface BuildCellProps {
  row: BuildRow;
  path: string;
}

interface ConsumersTableProps {
  consumers: readonly RegistryConsumer[];
}

export type { BuildCellProps, BuildsTableProps, ConsumersTableProps };
