/* @layer renderer-app @kind types */
import type { IconName, MenuGroup, TagCategoryColor, TypeMarkTone } from '@drizztdourden08/tessera';
import type { MouseEvent, ReactNode } from 'react';
import type { ItemRef, ThreadChoice } from '../../data/kanban';
import type { WithId } from '../../data/react';
import type { KindCollection } from '../../data/registry';
import type { AvatarSource } from '../../avatar';

type HeadSize = 'xs' | 'sm';

type HeadThreads = readonly WithId<ThreadChoice>[];

interface RecordRef {
  kind: string;
  collection: KindCollection;
  code: string;
  id: string;
  board: string | null;
  tab?: string;
}

interface TextPart {
  text: string;
  ref: RecordRef | null;
}

interface RecordTarget {
  href: string;
  go: () => void;
}

interface RecordLinkProps {
  id: string;
  board?: string;
  tab?: string;
  children?: ReactNode;
}

type ItemLookup = (stored: string) => { type?: string } | undefined;

interface ShownCodeProps {
  id: string;
  board?: string;
}

interface DepRecordLinkProps {
  id: string;
}

interface RecordAnchorProps {
  target: RecordRef | null;
  label: ReactNode;
  title: string;
}

interface LinkedTextProps {
  text: string;
  board?: string;
}

interface AssigneeLook {
  label: string;
  tip: string;
  initials?: string;
  icon?: IconName;
  color?: TagCategoryColor;
  lead?: string;
  owner: boolean;
}

interface HeadSelection {
  selecting: boolean;
  selected: boolean;
  onToggle: () => void;
  onMark: (shift: boolean) => void;
}

interface RecordHeadProps {
  kind: string;
  code: string;
  assignee: string;
  boardKey: string;
  threads: HeadThreads;
  menu: readonly MenuGroup[];
  menuLabel: string;
  title?: ReactNode;
  mark?: ReactNode;
  size?: HeadSize;
  selection?: HeadSelection;
}

interface KindMark {
  color?: TagCategoryColor;
  tone?: TypeMarkTone;
}

interface TypeIconProps {
  kind: string;
  size?: HeadSize;
}

interface ItemTypeMarkProps {
  item: ItemRef;
  size?: HeadSize;
}

interface AssigneeMarkProps {
  assignee: string;
  boardKey: string;
  threads: HeadThreads;
  size?: HeadSize;
}

interface LeadMarkProps {
  source: AvatarSource;
  label: string;
  size?: HeadSize;
}

interface RecordMenuProps {
  groups: readonly MenuGroup[];
  label: string;
}

interface RecordCardProps {
  head: RecordHeadProps;
  title: string;
  openLabel: string;
  onOpen: () => void;
  signals?: ReactNode;
  foot?: ReactNode;
  attrs?: Readonly<Record<string, string>>;
  onClickCapture?: (event: MouseEvent) => void;
}

interface ActionDef {
  id: string;
  label: string;
  group: 'open' | 'act' | 'soft';
  tone?: 'danger';
}

type ActionHandlers = Readonly<Record<string, (() => void) | undefined>>;

interface RecordPaneState {
  ref: RecordRef | null;
  tab: string | null;
  canBack: boolean;
  open: (code: string, tab?: string) => void;
  step: (code: string) => void;
  setTab: (tab: string) => void;
  back: () => void;
  close: () => void;
}

interface OpenOfHref {
  code: string;
  tab?: string;
}

interface LegacyOpen {
  code: string;
  tab: string | null;
}

interface RecordGrammar {
  token: RegExp;
  checks: Readonly<Record<string, RegExp>>;
}

interface RecordOrder {
  ids: readonly string[];
  set: (ids: readonly string[]) => void;
}

interface RecordPaneScopeProps {
  children: ReactNode;
}

export type {
  ActionDef, ActionHandlers, AssigneeLook, AssigneeMarkProps, DepRecordLinkProps, ItemLookup, ItemTypeMarkProps, KindMark, LeadMarkProps, LegacyOpen, LinkedTextProps,
  OpenOfHref, RecordAnchorProps, RecordCardProps, RecordHeadProps, RecordLinkProps, RecordMenuProps, RecordOrder, RecordGrammar, RecordPaneScopeProps,
  RecordPaneState, RecordRef, RecordTarget, ShownCodeProps, TextPart, TypeIconProps,
};
