/* @layer renderer-app @kind types */
import type { OverflowRowItem, SortDirection, TabItem } from '@drizztdourden08/tessera';
import type { ReactNode } from 'react';
import type { Option } from '../shared';

interface BoardTabsRowProps {
  tabs: TabItem[];
  activeTab: string;
  onTabChange: (tab: string) => void;
}

interface BoardTabsProps extends BoardTabsRowProps {
  tight: boolean;
}

interface FiltersToggleProps {
  open: boolean;
  count: number;
  onToggle: () => void;
}

interface ToolToggleProps {
  label: string;
  pressed: boolean;
  count?: number;
  onToggle: () => void;
}

interface TabFilterRowProps {
  tab: string;
  count: number;
  children: ReactNode;
}

interface MultiField<F> {
  kind: 'multi';
  path: string;
  label: string;
  options: readonly Option[];
  read: (filters: F) => readonly string[];
  write: (values: string[]) => Partial<F>;
}

interface SingleField<F> {
  kind: 'single';
  path: string;
  label: string;
  options: readonly Option[];
  read: (filters: F) => string;
  write: (value: string) => Partial<F>;
}

interface FlagField<F> {
  kind: 'flag';
  path: string;
  label: string;
  read: (filters: F) => boolean;
  write: (on: boolean) => Partial<F>;
}

type TabField<F> = MultiField<F> | SingleField<F> | FlagField<F>;

interface TabFilterBarProps<F extends { query: string }> {
  tab: string;
  boardKey: string;
  fields: readonly TabField<F>[];
  filters: F;
  patchFilters: (values: Partial<F>) => void;
  placeholder: string;
  /** False on a tab without threads (Releases): no Focus chip. */
  focus?: boolean;
}

interface TabFieldControlProps<F> {
  field: TabField<F>;
  filters: F;
  patch: (values: Partial<F>) => void;
}

interface TabFieldControlsProps<F> {
  fields: readonly TabField<F>[];
  filters: F;
  patch: (values: Partial<F>) => void;
}

interface FlagFilters {
  needs: boolean;
  conflict: boolean;
  archived: boolean;
}

interface ViewChoice<Id extends string> {
  id: Id;
  label: string;
}

interface GroupPart<G extends string> {
  options: readonly ViewChoice<G>[];
  value: G;
  onChange: (group: G) => void;
}

interface SortPart<O extends string> {
  options: readonly ViewChoice<O>[];
  order: O;
  direction: SortDirection;
  onChange: (order: O, direction: SortDirection) => void;
}

interface TabGroupSortProps<G extends string, O extends string> {
  group?: GroupPart<G>;
  sort: SortPart<O>;
}

interface ViewState<G extends string, O extends string> {
  prefs: { group: G; order: O; direction: SortDirection | null };
  setGroup: (group: G) => void;
  setOrder: (order: O) => void;
  setDirection: (direction: SortDirection) => void;
}

interface TabToolsEntry {
  tab: string;
  items: readonly OverflowRowItem[];
}

interface TabFilterInfo {
  tab: string;
  count: number;
}

interface RegistryEntry<T> {
  key: string;
  value: T;
}

interface Registry<T> {
  set: (key: string, value: T) => void;
  clear: (key: string) => void;
  subscribe: (listener: () => void) => () => void;
  get: () => readonly RegistryEntry<T>[];
}

interface TabViewSetup<G extends string, O extends string, F> {
  groups: readonly G[];
  orders: readonly O[];
  group: G;
  order: O;
  direction: SortDirection;
  filters: F;
}

interface TabView<G extends string, O extends string, F> {
  group: G;
  setGroup: (group: G) => void;
  order: O;
  direction: SortDirection;
  setSort: (order: O, direction: SortDirection) => void;
  filters: F;
  patchFilters: (values: Partial<F>) => void;
}

export type {
  BoardTabsProps, BoardTabsRowProps, FiltersToggleProps, FlagField, FlagFilters, GroupPart, MultiField, Registry, RegistryEntry, SingleField, SortPart,
  TabField, TabFieldControlProps, TabFieldControlsProps, TabFilterBarProps, TabFilterInfo, TabFilterRowProps, TabGroupSortProps, TabToolsEntry, TabView,
  TabViewSetup, ToolToggleProps, ViewChoice, ViewState,
};
