/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { RecordRow, recordMenuGroups, useRecordPane } from '../../../../../shell';
import { requestTitle } from '../../../../../data/worktrees';
import { requestChips, requestCode, requestNote } from '../model';
import { REQUEST_KIND } from '../worktrees.constants';
import type { RequestRowProps } from '../worktrees-props.type';

const RequestRow = (props: RequestRowProps) => {
  const { request, boardKey, threads } = props;
  const pane = useRecordPane();
  const code = requestCode(request);
  const open = () => pane.open(code);
  const note = requestNote(request);
  return (
    <Flex direction="column" data-swap-item>
      <RecordRow
        head={{ kind: REQUEST_KIND, code, assignee: request.by, boardKey, threads, menu: recordMenuGroups(REQUEST_KIND, code, { open }), menuLabel: `More actions for ${code}` }}
        title={requestTitle(request)}
        chips={requestChips(request)}
        time={request.at}
        trailing={note ? <Text variant="caption" tone="muted" clamp={1}>{note}</Text> : undefined}
        onOpen={open}
      />
    </Flex>
  );
};

export { RequestRow };
