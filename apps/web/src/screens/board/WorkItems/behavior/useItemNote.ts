/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useState } from 'react';
import type { WorkItem } from '../../../../data/types';
import { noteFields, replyProblem } from '../../owner-posts';
import type { NoteKind, OwnerPoster } from '../../owner-posts';
import type { ItemNote } from '../WorkItems.type';

const saidOf = (kind: NoteKind, thread: string): string =>
  kind === 'decision' ? `Decision queued for ${thread}.` : `Note sent to ${thread}.`;

const useItemNote = (item: WorkItem, poster: OwnerPoster): ItemNote => {
  const [kind, setKind] = useState<NoteKind>('decision');
  const [text, setText] = useState('');
  const key = `compose:${item.id}`;
  const send = () => {
    const body = text.trim();
    const problem = replyProblem(body, kind);
    if (problem) {
      toast({ variant: 'danger', message: problem });
      return;
    }
    void poster.submit({ key, fields: noteFields(item, kind, body), said: saidOf(kind, item.thread) }).then((taken) => taken && setText(''));
  };
  return { kind, text, busy: poster.isBusy(key), setKind, setText, send };
};

export { useItemNote };
