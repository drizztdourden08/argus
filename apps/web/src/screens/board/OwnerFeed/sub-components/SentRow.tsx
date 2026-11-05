/* @layer renderer-app @kind component */
import type { SentRowProps } from '../OwnerFeed.type';
import { EntryRow } from './EntryRow';

const SentRow = (props: SentRowProps) => <EntryRow type="sent" entry={props.entry} ctx={props.ctx} />;

export { SentRow };
