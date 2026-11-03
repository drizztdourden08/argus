/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { RecordRow } from '../../../../../shell';
import { formatAgo } from '../../../../../format';
import { requestChips, requestNote, requestTitle } from '../model';
import { WORKTREE_KIND } from '../worktrees.constants';
import type { RequestRowProps } from '../worktrees-props.type';

const RequestRow = (props: RequestRowProps) => {
  const { request, boardKey, threads, now } = props;
  const note = requestNote(request);
  return (
    <RecordRow
      head={{ kind: WORKTREE_KIND, code: request.id, assignee: request.by, boardKey, threads, menu: [], menuLabel: `More actions for ${request.id}` }}
      title={requestTitle(request)}
      chips={requestChips(request)}
      age={request.at ? formatAgo(request.at, now) : undefined}
      trailing={note ? <Text variant="caption" tone="muted" clamp={1}>{note}</Text> : undefined}
    />
  );
};

export { RequestRow };
