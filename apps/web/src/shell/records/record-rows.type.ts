/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { RecordHeadProps } from './records.type';

interface RowChip {
  id: string;
  text: string;
  title?: string;
  link?: string;
  code?: boolean;
}

interface RowChipsProps {
  chips: readonly RowChip[];
  board: string;
}

interface RowChipTagProps {
  chip: RowChip;
  board: string;
}

interface LinkedTitleProps {
  text: string;
  board: string;
  onOpen: () => void;
  weight?: number;
  clamp?: number;
  label?: string;
}

interface RecordRowProps {
  head: Omit<RecordHeadProps, 'selection' | 'size'>;
  title: string;
  onOpen: () => void;
  chips?: readonly RowChip[];
  age?: string;
  trailing?: ReactNode;
}

interface RecordGroupProps {
  header: ReactNode;
  count: number;
  flagged?: boolean;
  defaultOpen?: boolean;
  label: string;
  children: ReactNode;
}

interface RecordGroupData<T> {
  key: string;
  rows: T[];
}

interface RecordGroups<T> {
  groups: RecordGroupData<T>[];
  none: T[];
  closed: RecordGroupData<T>[];
}

interface GroupInfo {
  rank: number;
  closed: boolean;
}

export type {
  GroupInfo, LinkedTitleProps, RecordGroupData, RecordGroupProps, RecordGroups, RecordRowProps, RowChip, RowChipTagProps, RowChipsProps,
};
