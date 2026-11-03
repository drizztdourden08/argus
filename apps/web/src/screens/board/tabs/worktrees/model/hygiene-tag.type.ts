/* @layer renderer-app @kind types */
import type { TagLook, TagProps } from '@drizztdourden08/tessera';

interface HygieneTag {
  text: string;
  tip: string;
  look: TagLook;
  edge?: TagProps['edge'];
}

export type { HygieneTag };
