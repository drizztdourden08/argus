/* @layer renderer-app @kind component */
import { RecordCard, recordMenuGroups, useRecordPane } from '../../../../../shell';
import { copyText } from '../../../shared';
import { usePrChecks } from '../behavior/usePrChecks';
import { CHANGE_KIND } from '../changes.constants';
import type { PrCardProps } from '../changes.type';
import { PrFoot } from './PrFoot';
import { PrSignals } from './PrSignals';

const PrCard = (props: PrCardProps) => {
  const { boardKey, pr, threads } = props;
  const pane = useRecordPane();
  const { data } = usePrChecks(boardKey, pr);
  const open = () => pane.open(pr.id);
  return (
    <RecordCard
      head={{
        kind: CHANGE_KIND,
        code: pr.id,
        assignee: pr.createdBy ?? '',
        boardKey,
        threads,
        menu: recordMenuGroups(CHANGE_KIND, pr.id, { open, copy: () => copyText(pr.id) }),
        menuLabel: `More actions for ${pr.id}`,
      }}
      title={pr.title}
      openLabel={`Open ${pr.id}: ${pr.title}`}
      onOpen={open}
      label={`Pull request ${pr.id}: ${pr.title}`}
      edge="border"
      attrs={{ 'data-pr': pr.id, 'data-swap-item': '' }}
      signals={<PrSignals pr={pr} runs={data} />}
      foot={<PrFoot pr={pr} runs={data} />}
    />
  );
};

export { PrCard };
