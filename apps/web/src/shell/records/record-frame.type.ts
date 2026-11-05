/* @layer renderer-app @kind types */
import type { ReactNode, MouseEvent } from 'react';
import type { HeadSelection } from './records.type';

type FrameDensity = 'card' | 'row';

interface RecordFrameProps {
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  name: string;
  kind?: string;
  icon?: ReactNode;
  selection?: HeadSelection;
  code?: ReactNode;
  title?: ReactNode;
  content?: ReactNode;
  menu?: ReactNode;
  density?: FrameDensity;
  attrs?: Readonly<Record<string, string>>;
  onClickCapture?: (event: MouseEvent) => void;
  children?: ReactNode;
}

interface FrameCodeProps {
  children: ReactNode;
}

interface FrameLeadProps {
  kind?: string;
  icon?: ReactNode;
  selection?: HeadSelection;
  name: string;
}

export type { FrameCodeProps, FrameLeadProps, RecordFrameProps };
