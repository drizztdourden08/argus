/* @layer renderer-app @kind component */
import { EntryOverview } from '../EntryOverview';
import { FactsOverview } from '../FactsOverview';
import type { RecordBodyProps } from '../pane.type';

const RecordOverviewTab = (props: RecordBodyProps) => (props.data.entry ? <EntryOverview {...props} /> : <FactsOverview {...props} />);

export { RecordOverviewTab };
