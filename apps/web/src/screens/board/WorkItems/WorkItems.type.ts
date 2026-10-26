/* @layer renderer-app @kind types */
import type { Criterion, WorkItem } from '../../../data/types';
import type { ItemNeeds, NeedCount } from '../feed-model';
import type { NoteKind, OwnerPoster } from '../owner-posts';

type ItemStatusKey = 'todo' | 'doing' | 'review' | 'done' | 'dropped';

interface WorkItemsProps {
  boardKey: string;
  items: WorkItem[];
  needs: ItemNeeds;
}

interface WorkItemCardProps {
  item: WorkItem;
  need: NeedCount;
  poster: OwnerPoster;
}

interface ItemHeaderProps {
  item: WorkItem;
  need: NeedCount;
}

interface NeedTagsProps {
  need: NeedCount;
}

interface CriteriaListProps {
  criteria: Criterion[];
  label: string;
}

interface ItemNoteFormProps {
  item: WorkItem;
  poster: OwnerPoster;
}

interface ItemNote {
  kind: NoteKind;
  text: string;
  busy: boolean;
  setKind: (kind: NoteKind) => void;
  setText: (text: string) => void;
  send: () => void;
}

export type {
  CriteriaListProps,
  ItemHeaderProps,
  ItemNote,
  ItemNoteFormProps,
  ItemStatusKey,
  NeedTagsProps,
  WorkItemCardProps,
  WorkItemsProps,
};
