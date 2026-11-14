/* @layer renderer-app @kind types */
import type { IconName, MenuGroup, TagCategoryColor, TypeMarkTone } from '@drizztdourden08/tessera';
import type { MouseEvent, ReactNode } from 'react';
import type { ItemRef, ThreadChoice } from '../../data/kanban';
import type { WithId } from '../../data/react';
import type { KindCollection } from '../../data/registry';
import type { OwnerProfile } from '../../owner';
import type { AvatarSource } from '../../avatar';
import type { RecordFrameProps } from './record-frame.type';

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

interface AgentName {
  id: string;
  name?: string | null;
}

type Threads = readonly WithId<ThreadChoice>[];

interface LookExtras {
  owner?: OwnerProfile;
  agents?: readonly AgentName[];
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
  mark?: ReactNode;
  size?: HeadSize;
  selection?: HeadSelection;
}

type HeadFrameKeys = 'title' | 'content' | 'density' | 'defaultOpen' | 'open' | 'onOpenChange' | 'attrs' | 'onClickCapture' | 'children' | 'edge' | 'edgeTone' | 'variant' | 'fill' | 'className' | 'label' | 'onKeyDown' | 'onActivate';

interface RecordHeadViewProps extends RecordHeadProps, Pick<RecordFrameProps, HeadFrameKeys> {}

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
  defaultOpen?: boolean;
  content?: ReactNode;
  edge?: RecordFrameProps['edge'];
  edgeTone?: RecordFrameProps['edgeTone'];
  variant?: RecordFrameProps['variant'];
  fill?: RecordFrameProps['fill'];
  className?: string;
  label?: string;
  onKeyDown?: RecordFrameProps['onKeyDown'];
  onActivate?: RecordFrameProps['onActivate'];
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
  ActionDef, ActionHandlers, AgentName, AssigneeLook, AssigneeMarkProps, DepRecordLinkProps, ItemLookup, ItemTypeMarkProps, KindMark, LeadMarkProps, LegacyOpen, LinkedTextProps, LookExtras, Threads,
  HeadSelection, OpenOfHref, RecordAnchorProps, RecordCardProps, RecordHeadProps, RecordHeadViewProps, RecordLinkProps, RecordMenuProps, RecordOrder, RecordGrammar, RecordPaneScopeProps,
  RecordPaneState, RecordRef, RecordTarget, ShownCodeProps, TextPart, TypeIconProps,
};
