/* @layer renderer-app @kind constants */
import type { CardProps } from '@drizztdourden08/tessera';
import type { FeedKind } from '../feed-model';
import type { FeedFilter } from './OwnerFeed.type';

const FILTER_STORAGE_KEY = 'coord.feed';

const FILTER_OPTIONS: { value: FeedFilter; label: string }[] = [
  { value: 'all', label: 'All' },
  { value: 'you', label: 'Needs you' },
  { value: 'log', label: 'Log' },
];

const LOG_START = 12;

const LOG_STEP = 25;

const CARD_TONES: Record<FeedKind, CardProps['tone']> = {
  decision: 'danger',
  question: 'warning',
  message: 'warning',
  action: 'warning',
  test: 'warning',
  bundle: 'warning',
  notice: 'info',
};

export { CARD_TONES, FILTER_OPTIONS, FILTER_STORAGE_KEY, LOG_START, LOG_STEP };
