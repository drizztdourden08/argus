/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { ItemRef, ThreadChoice } from '../../../data/kanban';
import type { WithId } from '../../../data/react';

type GroupThreads = readonly WithId<ThreadChoice>[];

interface ItemGroup<T> {
  item: ItemRef;
  rows: T[];
}

interface ItemGroups<T> {
  groups: ItemGroup<T>[];
  none: T[];
  closed: ItemGroup<T>[];
}

interface GroupsData {
  ready: boolean;
  items: ItemRef[];
  threads: GroupThreads;
}

interface ItemGroupHeaderProps {
  item: ItemRef;
  boardKey: string;
  threads: GroupThreads;
}

interface ItemGroupSectionsProps<T> {
  groups: ItemGroups<T>;
  boardKey: string;
  threads: GroupThreads;
  label: string;
  noneTitle: string;
  noneFlag?: boolean;
  lead?: ReactNode;
  tail?: ReactNode;
  children: (rows: T[]) => ReactNode;
}

interface ClosedGroupsProps<T> {
  groups: readonly ItemGroup<T>[];
  boardKey: string;
  threads: GroupThreads;
  label: string;
  children: (rows: T[]) => ReactNode;
}

export type { ClosedGroupsProps, GroupsData, GroupThreads, ItemGroup, ItemGroupHeaderProps, ItemGroups, ItemGroupSectionsProps };
