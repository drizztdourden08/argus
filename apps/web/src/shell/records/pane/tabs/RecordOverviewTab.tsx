/* @layer renderer-app @kind component */
import { EntryOverview } from '../EntryOverview';
import { FactsOverview } from '../FactsOverview';
import type { RecordBodyProps } from '../pane.type';
import { TestCaseOverviewTab } from './TestCaseOverviewTab';

const RecordOverviewTab = (props: RecordBodyProps) => {
  if (props.data.kind.collection === 'tests') return <TestCaseOverviewTab {...props} />;
  return props.data.entry ? <EntryOverview {...props} /> : <FactsOverview {...props} />;
};

export { RecordOverviewTab };
