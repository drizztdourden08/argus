/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { cancelDep, depColor, isLive } from '../../../../data/deps';
import { kindOf } from '../../../../data/registry';
import { Age, DepTag } from '../../../../shell';
import { shownCode } from '../../../../shell/records';
import { leadOf, OverviewRow } from '../../records';
import { DEP_KIND, DEP_PANE_TAB, TRAIL_MIN } from '../deps.constants';
import type { DepRowProps } from '../deps.type';
import { changedAt, trailLength } from '../model';
import { DepEndCell } from './DepEndCell';
import { DepWaitCell } from './DepWaitCell';

const DepRow = (props: DepRowProps) => {
  const { doc, all, now } = props;
  const kind = kindOf(DEP_KIND);
  const trail = trailLength(doc, all);
  const chips = trail >= TRAIL_MIN ? [{ id: 'trail', text: `trail ${trail}`, title: `${trail} requests in a chain` }] : [];
  return (
    <OverviewRow
      target={{ kind: DEP_KIND, code: kind ? shownCode(kind, doc.id) : doc.id, project: doc.from.project, open: doc.from.item, tab: DEP_PANE_TAB }}
      assignee={leadOf(doc.from.project)}
      title={doc.title}
      chips={chips}
      handlers={isLive(doc.state) ? { cancel: () => void cancelDep(doc.id) } : undefined}
      trailing={
        <Flex gap="xs" align="center">
          <DepEndCell end={doc.from} />
          <Text variant="caption" tone="muted">waits on</Text>
          <DepEndCell end={doc.to} />
          <DepTag color={depColor(doc.state)} title={doc.reason ?? doc.title}>{doc.state}</DepTag>
          <DepWaitCell doc={doc} now={now} />
          <Text variant="caption" tone="muted"><Age value={changedAt(doc)} /></Text>
        </Flex>
      }
    />
  );
};

export { DepRow };
