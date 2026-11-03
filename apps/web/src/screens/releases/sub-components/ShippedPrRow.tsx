/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { shortCommit } from '../../../format';
import { recordMenuGroups, RecordRow, useRecordPane } from '../../../shell';
import { CHANGE_KIND } from '../releases.constants';
import type { ShippedPrRowProps } from '../releases.type';

const ShippedPrRow = (props: ShippedPrRowProps) => {
  const { boardKey, pr, local } = props;
  const pane = useRecordPane();
  if (!local) return <Text variant="caption" tone="muted">{`Pull request #${pr.number}, merge ${shortCommit(pr.commit)}`}</Text>;
  const code = local.id;
  const open = () => pane.open(code);
  return (
    <RecordRow
      head={{ kind: CHANGE_KIND, code, assignee: local.createdBy ?? '', boardKey, threads: [], menu: recordMenuGroups(CHANGE_KIND, code, { open }), menuLabel: `More actions for ${code}` }}
      title={local.title}
      chips={[{ id: 'status', text: local.status }]}
      onOpen={open}
    />
  );
};

export { ShippedPrRow };
