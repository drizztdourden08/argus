/* @layer renderer-app @kind types */
import type { Entry } from '../../../data/types';
import type { AskMode, FeedCard } from '../feed-model';
import type { NoteKind } from '../owner-posts';
import type { EntryBodyProps, FeedContext } from './OwnerFeed.type';

type AskFeedCard = Extract<FeedCard, { data: Entry }>;

interface AskLines {
  title: string;
  rest: string;
}

interface AskCopy {
  kind: NoteKind;
  label: string;
  verb: string;
  fold: string;
}

interface AskGroupProps {
  heading: string;
  cards: AskFeedCard[];
  ctx: FeedContext;
}

interface AskCardProps {
  card: AskFeedCard;
  ctx: FeedContext;
}

interface AskFormProps extends EntryBodyProps {
  mode: AskMode;
  quick?: boolean;
}

interface ContextPreviewProps extends EntryBodyProps {
  onMore: () => void;
}

export type { AskCardProps, AskCopy, AskFeedCard, AskFormProps, AskGroupProps, AskLines, ContextPreviewProps };
