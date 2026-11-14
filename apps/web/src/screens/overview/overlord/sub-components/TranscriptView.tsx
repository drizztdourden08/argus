/* @layer renderer-app @kind component */
import { Box, Callout, EmptyState } from '@drizztdourden08/tessera';
import { Panel } from '../../../../shell';
import { EMPTY_TRANSCRIPT } from '../overlord.constants';
import type { TranscriptViewProps } from '../overlord.type';
import { TalkRowView } from './TalkRowView';
import { ToolsRow } from './ToolsRow';

const TranscriptView = (props: TranscriptViewProps) => {
  const { rows, problem } = props;
  const newestFirst = [...rows].reverse();
  return (
    <Panel title="Transcript">
      {problem && <Callout tone="danger">{problem}</Callout>}
      {rows.length === 0 && !problem && <EmptyState message={EMPTY_TRANSCRIPT} />}
      <Box className="overlord-transcript">
        {newestFirst.map((row) => (row.kind === 'tools' ? <ToolsRow key={row.key} names={row.names} /> : <TalkRowView key={row.key} row={row} />))}
      </Box>
    </Panel>
  );
};

export { TranscriptView };
