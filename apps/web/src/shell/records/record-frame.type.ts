/* @layer renderer-app @kind types */
import type { CardEdge, CardTone, CardVariant } from '@drizztdourden08/tessera';
import type { KeyboardEvent, MouseEvent, ReactNode } from 'react';
import type { HeadSelection } from './records.type';

type FrameDensity = 'card' | 'row' | 'compact';

interface RecordFrameProps {
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  name: string;
  label?: string;
  kind?: string;
  icon?: ReactNode;
  selection?: HeadSelection;
  code?: ReactNode;
  title?: ReactNode;
  content?: ReactNode;
  menu?: ReactNode;
  density?: FrameDensity;
  chevronOnIcon?: boolean;
  edge?: CardEdge;
  edgeTone?: CardTone;
  variant?: CardVariant;
  fill?: boolean;
  className?: string;
  attrs?: Readonly<Record<string, string>>;
  onClickCapture?: (event: MouseEvent) => void;
  onKeyDown?: (event: KeyboardEvent<HTMLElement>) => void;
  onActivate?: () => void;
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
