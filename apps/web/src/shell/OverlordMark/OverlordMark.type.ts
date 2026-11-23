/* @layer renderer-app @kind types */
import type { OverlordActivity } from '../../data/overlord';

interface OverlordMarkProps {
  activity: OverlordActivity;
  size: 'sm' | 'lg';
  className?: string;
}

export type { OverlordMarkProps };
