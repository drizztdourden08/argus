/* @layer renderer-app @kind types */
import type { TagLook } from '@drizztdourden08/tessera';
import type { ReactNode } from 'react';
import type { RecordHeadProps, RecordTimeValue } from './records.type';

interface RowChip {
  id: string;
  text: string;
  title?: string;
  link?: string;
  code?: boolean;
  look?: TagLook;
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
  onOpen?: () => void;
  chips?: readonly RowChip[];
  time?: RecordTimeValue;
  compact?: boolean;
  trailing?: ReactNode;
  body?: ReactNode;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
}

interface CardTitleProps {
  title: string;
  board: string;
  label: string;
  onOpen: () => void;
  clamp: number;
}

interface RowTitleProps {
  title: string;
  board: string;
  linked: boolean;
  onOpen?: () => void;
  label: string;
}

interface RecordGroupProps {
  header: ReactNode;
  count: number;
  flagged?: boolean;
  defaultOpen?: boolean;
  label: string;
  children: ReactNode;
}

interface RecordGroupHeaderProps {
  kind: string;
  code: string;
  title: string;
  boardKey: string;
  onOpen: () => void;
  tag?: { text: string; title?: string };
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
  CardTitleProps, GroupInfo, LinkedTitleProps, RecordGroupData, RecordGroupHeaderProps, RecordGroupProps, RecordGroups, RecordRowProps, RowChip, RowChipTagProps, RowChipsProps, RowTitleProps,
};
