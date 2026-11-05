/* @layer renderer-app @kind types */
import type { FieldDescriptor, FilterClause, TabItem } from '@drizztdourden08/tessera';
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

export type {
  BoardTabsRowProps, ChoiceField, ClauseField, ClauseFilterRowProps, ClauseFilters, FiltersToggleProps, FlagField, PickField, PortalProps, TabFiltersProps,
};
