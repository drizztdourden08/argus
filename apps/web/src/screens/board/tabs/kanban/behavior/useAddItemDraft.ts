/* @layer renderer-app @kind logic */
import { useState } from 'react';
import { threadChoices } from '../../../shared';
import { STARTING_STATUS, listedThreads } from '../model';
import type { AddPreset } from '../model';
import type { AddItemDraft, Threads } from '../kanban.type';

const useAddItemDraft = (threads: Threads, preset: AddPreset): AddItemDraft => {
  const [picked, setThread] = useState<string | null>(preset.thread ?? null);
  const thread = picked ?? listedThreads(threads)[0] ?? threadChoices(threads)[0]?.value ?? '';
  return {
    thread,
    setThread,
    itemOf: (title) => ({ title, thread, status: preset.status ?? STARTING_STATUS, ...(preset.subject ? { subject: preset.subject } : {}) }),
  };
};

export { useAddItemDraft };
