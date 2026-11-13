/* @layer renderer-app @kind logic */
import type { TabField } from '../../BoardTabsRow';
import type { Option } from '../../shared';
import { FEED_KIND_OPTIONS } from '../OwnerFeed.constants';
import type { FeedFilters } from '../OwnerFeed.type';

const feedFields = (authors: readonly Option[]): TabField<FeedFilters>[] => [
  { kind: 'multi', path: 'authors', label: 'Author', options: authors, read: (filters) => filters.authors, write: (values) => ({ authors: values }) },
  { kind: 'multi', path: 'kinds', label: 'Kind', options: FEED_KIND_OPTIONS, read: (filters) => filters.kinds, write: (values) => ({ kinds: values }) },
];

export { feedFields };
