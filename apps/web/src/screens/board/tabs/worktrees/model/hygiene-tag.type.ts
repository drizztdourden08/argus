/* @layer renderer-app @kind types */
import type { TagEdge, TagLook } from '@drizztdourden08/tessera';

interface HygieneTag {
  text: string;
  tip: string;
  look: TagLook;
  edge?: TagEdge;
}

export type { HygieneTag };
