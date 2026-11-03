/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { RecordRow, recordMenuGroups, useRecordPane } from '../../../../../shell';
import { formatAgo } from '../../../../../format';
import { requestTitle } from '../../../../../data/worktrees';
import { requestChips, requestCode, requestNote } from '../model';
import { REQUEST_KIND } from '../worktrees.constants';
import type { RequestRowProps } from '../worktrees-props.type';

const RequestRow = (props: RequestRowProps) => {
  const { request, boardKey, threads, now } = props;
  const pane = useRecordPane();
  const code = requestCode(request);
  const open = () => pane.open(code);
  const note = requestNote(request);
  return (
    <RecordRow
      head={{ kind: REQUEST_KIND, code, assignee: request.by, boardKey, threads, menu: recordMenuGroups(REQUEST_KIND, code, { open }), menuLabel: `More actions for ${code}` }}
      title={requestTitle(request)}
      chips={requestChips(request)}
      age={request.at ? formatAgo(request.at, now) : undefined}
      trailing={note ? <Text variant="caption" tone="muted" clamp={1}>{note}</Text> : undefined}
      onOpen={open}
    />
  );
};

export { RequestRow };
