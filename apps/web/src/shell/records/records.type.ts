/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';

type RecordKind = 'item' | 'entry' | 'todo' | 'test' | 'agent';

interface RecordRef {
  kind: RecordKind;
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
  children?: ReactNode;
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

export type { DepRecordLinkProps, LinkedTextProps, RecordAnchorProps, RecordKind, RecordLinkProps, RecordRef, RecordTarget, TextPart };
