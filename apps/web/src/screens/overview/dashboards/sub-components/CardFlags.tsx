/* @layer renderer-app @kind component */
import { Inline, Tag } from '@drizztdourden08/tessera';
import { BAD_LOOK, WARN_LOOK } from '../dashboards.constants';
import type { CardCountersProps } from '../dashboards.type';
import { attentionOf } from '../model';

const CardFlags = (props: CardCountersProps) => {
  const { card } = props;
  const attention = attentionOf(card);
  return (
    <Inline gap="xs" align="center" wrap>
      {attention.decisions > 0 && <Tag {...WARN_LOOK} title="open decisions waiting for you">{`${attention.decisions} decision${attention.decisions > 1 ? 's' : ''}`}</Tag>}
      {attention.failing > 0 && <Tag {...BAD_LOOK} title="tests failing">{`${attention.failing} failing`}</Tag>}
      {attention.blocked > 0 && <Tag {...BAD_LOOK} title="blocked threads">{`${attention.blocked} blocked`}</Tag>}
    </Inline>
  );
};

export { CardFlags };
