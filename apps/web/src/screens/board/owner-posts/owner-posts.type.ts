/* @layer renderer-app @kind types */
import type { OwnerFields } from '../../../data/posts';

interface PostSpec {
  key: string;
  fields: OwnerFields;
  said: string;
}

interface OwnerPoster {
  isBusy: (key: string) => boolean;
  submit: (spec: PostSpec) => Promise<boolean>;
}

interface PostTarget {
  id: string;
  item?: string | null;
  thread?: string | null;
}

type NoteKind = 'decision' | 'followup';

type ReplyKind = NoteKind | 'ack' | 'done';

export type { NoteKind, OwnerPoster, PostSpec, PostTarget, ReplyKind };
