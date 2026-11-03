/* @layer renderer-app @kind types */
import type { Entry } from '../../../data/types';

interface EntryDrawerProps {
  boardKey: string;
  id: string | null;
  onClose: () => void;
}

interface EntryFactsProps {
  entry: Entry;
  onOpenItem: (id: string) => void;
}

export type { EntryDrawerProps, EntryFactsProps };
