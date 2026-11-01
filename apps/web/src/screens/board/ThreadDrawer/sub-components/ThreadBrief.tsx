/* @layer renderer-app @kind component */
import { EditableMarkdown } from '../../shared';
import { LIMITS, textProblem } from '../../tabs/kanban/model';
import type { ThreadBriefProps } from '../ThreadDrawer.type';
import { ThreadSection } from './ThreadSection';

const briefProblem = (text: string): string | null => textProblem('brief', text);

const ThreadBrief = (props: ThreadBriefProps) => {
  const { thread, ops } = props;
  return (
    <ThreadSection title="Brief">
      <EditableMarkdown
        noun="brief"
        value={thread.brief ?? ''}
        empty="No brief."
        limit={LIMITS.brief}
        problem={briefProblem}
        onSave={(value) => ops.setField(thread, 'brief', value)}
      />
    </ThreadSection>
  );
};

export { ThreadBrief };
