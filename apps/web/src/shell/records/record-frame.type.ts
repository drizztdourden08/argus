/* @layer renderer-app @kind types */
import type { ReactNode, MouseEvent } from 'react';
import type { HeadSelection } from './records.type';

type FrameDensity = 'card' | 'row';

interface FrameOpenState {
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
}

interface RecordFrameProps extends FrameOpenState {
  name: string;
  kind?: string;
  icon?: ReactNode;
  selection?: HeadSelection;
  code?: ReactNode;
  title?: ReactNode;
  compactTitle?: ReactNode;
  pinTitle?: boolean;
  content?: ReactNode;
  menu?: ReactNode;
  density?: FrameDensity;
  attrs?: Readonly<Record<string, string>>;
  onClickCapture?: (event: MouseEvent) => void;
  children?: ReactNode;
}

interface FrameToggleProps {
  open: boolean;
  name: string;
  onToggle: () => void;
}

interface FrameSlotProps {
  grow?: boolean;
  children: ReactNode;
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

interface FrameMarkProps {
  attrs?: Readonly<Record<string, string>>;
  children: ReactNode;
}

interface FrameHeadProps {
  open: boolean;
  onToggle: () => void;
  name: string;
  kind?: string;
  icon?: ReactNode;
  selection?: HeadSelection;
  code?: ReactNode;
  headTitle?: ReactNode;
  content?: ReactNode;
  menu?: ReactNode;
}

export type { FrameCodeProps, FrameHeadProps, FrameMarkProps, FrameLeadProps, FrameOpenState, FrameSlotProps, FrameToggleProps, RecordFrameProps };
