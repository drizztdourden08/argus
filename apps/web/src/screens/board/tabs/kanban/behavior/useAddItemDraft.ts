/* @layer renderer-app @kind logic */
import { useState } from 'react';
import type { ItemInput } from '../../../../../data/kanban';
import { LIMITS, STARTING_STATUS, listedThreads } from '../model';
import type { AddPreset } from '../model';
import type { AddItemDraft, Threads } from '../kanban.type';

const useAddItemDraft = (threads: Threads, preset: AddPreset, onCreate: (item: ItemInput) => void): AddItemDraft => {
  const [title, setTitle] = useState('');
  const [thread, setThread] = useState<string | null>(preset.thread ?? null);
  const [status, setStatus] = useState(preset.status ?? STARTING_STATUS);
  const chosen = thread ?? listedThreads(threads)[0] ?? '';
  const trimmed = title.trim();
  const reset = () => {
    setTitle('');
    setThread(preset.thread ?? null);
    setStatus(preset.status ?? STARTING_STATUS);
  };
  const ready = Boolean(trimmed && chosen && trimmed.length <= LIMITS.title);
  const submit = () => {
    if (!ready) return;
    onCreate({ title: trimmed, thread: chosen, status, ...(preset.subject ? { subject: preset.subject } : {}) });
    reset();
  };
  return { title, thread: chosen, status, setTitle, setThread, setStatus, ready, reset, submit };
};

export { useAddItemDraft };
