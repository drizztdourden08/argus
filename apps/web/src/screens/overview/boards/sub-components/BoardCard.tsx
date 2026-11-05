/* @layer renderer-app @kind component */
import { Badge, Box, Card, FoldText, Pressable, SignalStrip, Text } from '@drizztdourden08/tessera';
import { RecordCard, useRecordPane } from '../../../../shell/records';
import { BoardIcon } from '../../../../shell/BoardIcon';
import { attentionOf, cardTone } from '../../dashboards/model';
import { boardMenuGroups } from '../behavior/board-menu-groups';
import { leadDot } from '../behavior/lead-dot';
import { signalItems } from '../behavior/signal-items';
import { useBoardActions } from '../behavior/useBoardActions';
import { SIGNAL_MAX } from '../boards.constants';
import type { BoardCardProps } from '../boards.type';
import { boardSignals } from '../model';
import { BoardCardFoot } from './BoardCardFoot';
import './BoardCard.css';

const BoardCard = (props: BoardCardProps) => {
  const { record, card, live, now, send, sync, onLog } = props;
  const { run, handlers } = useBoardActions(record, card, send);
  const pane = useRecordPane();
  const dot = leadDot(record, live);
  const signals = signalItems(boardSignals(record, live), (tab) => pane.open(record.id, tab));
  const icon = <BoardIcon boardKey={record.key} icon={record.icon} size="sm" />;
  return (
    <Card
      fill
      className="board-card"
      edge="border"
      edgeTone={record.started && card && !record.archived ? cardTone(attentionOf(card)) : undefined}
      data-swap-item={record.id}
      data-started={record.started}
    >
      <RecordCard
        head={{
          kind: 'board',
          code: record.id,
          assignee: `lead:${record.key}`,
          boardKey: record.key,
          threads: [],
          menu: boardMenuGroups(record, record.started, handlers),
          menuLabel: `More actions for ${record.id}`,
          mark: record.started ? <Pressable onClick={handlers.open} aria-label={`Open the ${record.name} board`}>{icon}</Pressable> : icon,
          title: <Box as="span" title={dot.tip} data-lead-dot={record.lead}><Badge variant="dot" color={dot.color} /></Box>,
        }}
        title={record.name}
        openLabel={`Open ${record.id}: ${record.name}`}
        onOpen={() => pane.open(record.id)}
        signals={
          <>
            {record.goal && <FoldText lines={2}><Text variant="caption" tone="muted">{record.goal}</Text></FoldText>}
            {signals.length > 0 && <SignalStrip items={signals} max={SIGNAL_MAX} iconOnly aria-label="Signals" />}
          </>
        }
        foot={<BoardCardFoot record={record} card={card} now={now} send={send} run={run} sync={sync} onLog={onLog} />}
      />
    </Card>
  );
};

export { BoardCard };
