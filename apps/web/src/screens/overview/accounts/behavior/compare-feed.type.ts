/* @layer renderer-app @kind types */
import type { CompareDoc } from '../../../../data/types';

interface FeedState {
  data: CompareDoc | null;
  loading: boolean;
  error: string | null;
}

export type { FeedState };
