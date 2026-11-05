/* @layer renderer-app @kind types */
import type { FieldDescriptor, FilterClause, SortDirection, TabItem } from '@drizztdourden08/tessera';
import type { ReactNode } from 'react';
import type { Option } from '../shared';

interface BoardTabsRowProps {
  tabs: TabItem[];
  activeTab: string;
  onTabChange: (tab: string) => void;
}

interface FiltersToggleProps {
  open: boolean;
  count: number;
  onToggle: () => void;
}

interface TabFiltersProps {
  tab: string;
  count: number;
  children: ReactNode;
}

interface PortalProps {
  children: ReactNode;
}

interface ChoiceField<F> {
  kind: 'multi';
  path: string;
  label: string;
  options: readonly Option[];
  read: (filters: F) => readonly string[];
  write: (values: string[]) => Partial<F>;
}

interface PickField<F> {
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

type ClauseField<F> = ChoiceField<F> | PickField<F> | FlagField<F>;

interface ClauseFilters {
  schema: readonly FieldDescriptor[];
  clauses: readonly FilterClause[];
  onChange: (next: readonly FilterClause[]) => void;
  count: number;
}

interface ClauseFilterRowProps {
  boardKey: string;
  search: string;
  placeholder: string;
  onSearch: (query: string) => void;
  bar: ClauseFilters;
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

interface ViewState<G extends string, O extends string> {
  prefs: { group: G; order: O; direction: SortDirection | null };
  setGroup: (group: G) => void;
  setOrder: (order: O) => void;
  setDirection: (direction: SortDirection) => void;
}

interface TabViewMenuProps<G extends string, O extends string> {
  groups: readonly ViewChoice<G>[];
  orders: readonly ViewChoice<O>[];
  state: ViewState<G, O>;
  shown: (order: O, direction: SortDirection | null) => SortDirection;
}

interface TabFilterSetProps<F extends { query: string }> {
  tab: string;
  boardKey: string;
  fields: readonly ClauseField<F>[];
  filters: F;
  patchFilters: (values: Partial<F>) => void;
  placeholder: string;
}

export type {
  BoardTabsRowProps, ClauseField, ClauseFilterRowProps, ClauseFilters, FiltersToggleProps, FlagFilters, PortalProps, TabFilterSetProps, TabFiltersProps, TabViewMenuProps,
};
