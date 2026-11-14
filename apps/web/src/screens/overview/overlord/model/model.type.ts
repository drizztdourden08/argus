/* @layer renderer-app @kind types */
import type { OverlordMode } from '../../../../data/overlord';

interface ControlSet {
  start: boolean;
  restart: boolean;
  stop: boolean;
}

interface SpendLook {
  ratio: number;
  tone: 'success' | 'warning' | 'danger';
}

interface ModeStep {
  mode: OverlordMode;
  label: string;
  current: boolean;
  allowed: boolean;
  reason: string | null;
}

export type { ControlSet, ModeStep, SpendLook };
