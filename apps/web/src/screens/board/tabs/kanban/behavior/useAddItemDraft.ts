/* @layer renderer-app @kind logic */
import { useState } from 'react';
import type { ItemRef } from '../../../../../data/kanban';
import { requiresParent } from '../../../../../data/registry';
import { threadChoices } from '../../../shared';
import { DEFAULT_TYPE, STARTING_STATUS, addParentOptions, listedThreads } from '../model';
import type { AddPreset } from '../model';
import type { AddItemDraft, Threads } from '../kanban.type';

const useAddItemDraft = (threads: Threads, preset: AddPreset, items: ReadonlyMap<string, ItemRef>): AddItemDraft => {
  const [picked, setThread] = useState<string | null>(preset.thread ?? null);
  const [pickedParent, setParent] = useState<string>(preset.parent ?? '');
  const thread = picked ?? listedThreads(threads)[0] ?? threadChoices(threads)[0]?.value ?? '';
  const needsParent = requiresParent(DEFAULT_TYPE);
  return {
    thread,
    setThread,
    parent: pickedParent,
    setParent,
    parentOptions: addParentOptions(DEFAULT_TYPE, items),
    needsParent,
    ready: thread !== '' && (!needsParent || pickedParent !== ''),
    itemOf: (title) => ({ title, thread, status: preset.status ?? STARTING_STATUS, ...(pickedParent ? { parent: pickedParent } : {}), ...(preset.assignee ? { assignee: preset.assignee } : {}) }),
  };
};

export { useAddItemDraft };
