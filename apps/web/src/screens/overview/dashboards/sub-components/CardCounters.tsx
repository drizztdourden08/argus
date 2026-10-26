/* @layer renderer-app @kind component */
import { Inline, Tag } from '@drizztdourden08/tessera';
import { BAD_LOOK, CALM_LOOK } from '../dashboards.constants';
import type { CardCountersProps } from '../dashboards.type';
import { cardCounts } from '../model';

const CardCounters = (props: CardCountersProps) => {
  const { agents, items, notPassing, failing } = cardCounts(props.card);
  return (
    <Inline gap="xs" align="center" wrap>
      <Tag {...CALM_LOOK} title="agents working">{`agents ${agents}`}</Tag>
      <Tag {...CALM_LOOK} title="Kanban items outstanding">{`items ${items}`}</Tag>
      <Tag {...(failing > 0 ? BAD_LOOK : CALM_LOOK)} title="tests not passing">{`tests ${notPassing}`}</Tag>
    </Inline>
  );
};

export { CardCounters };
