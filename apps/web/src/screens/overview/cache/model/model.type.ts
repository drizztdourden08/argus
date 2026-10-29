/* @layer renderer-app @kind types */
import type { CacheRow } from '../../../../data/cache';

interface CacheTableRow {
  id: string;
  kind: string;
  name: string;
  version: string;
  builds: number;
  users: string;
  userCount: number;
  sizeBytes: number;
  lastUsedAt: string;
  item: CacheRow;
}

interface Said {
  variant: 'success' | 'danger';
  message: string;
}

export type { CacheTableRow, Said };
