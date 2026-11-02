/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { DepColor } from '../../data/deps';

interface DepTagProps {
  color: DepColor;
  title?: string;
  children: ReactNode;
}

export type { DepTagProps };
