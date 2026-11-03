/* @layer renderer-app @kind types */
import type { IconName } from '@drizztdourden08/tessera';
import type { ReactNode } from 'react';
import type { DepColor } from '../../data/deps';

interface DepTagProps {
  color: DepColor;
  icon?: IconName;
  title?: string;
  children: ReactNode;
}

export type { DepTagProps };
