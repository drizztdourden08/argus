/* @layer renderer-app @kind component */
import { FoldText, Text } from '@drizztdourden08/tessera';
import { agentsText } from '../behavior/agents-text';
import { leadLine } from '../behavior/lead-line';
import type { BoardBodyProps } from '../boards.type';
import { boardProgress, boardSignals } from '../model';
import { BoardLead } from './BoardLead';
import { BoardPlay } from './BoardPlay';
import { BoardProgress } from './BoardProgress';
import { BoardSignals } from './BoardSignals';

const BoardCardBody = (props: BoardBodyProps) => {
  const { record, card, live, now, run, onLog, onOpen } = props;
  const lead = <BoardLead line={leadLine(record, live, card, now)} agents={agentsText(record.counts)} />;
  const goal = record.goal && <FoldText lines={2}><Text variant="caption" tone="muted">{record.goal}</Text></FoldText>;
  if (!record.started) return <>{goal}{lead}</>;
  return (
    <>
      {goal}
      {lead}
      <BoardSignals signals={boardSignals(record, live)} onOpen={onOpen} />
      <BoardProgress progress={boardProgress(record)} />
      {card && <BoardPlay card={card} run={run} now={now} onLog={onLog} />}
    </>
  );
};

export { BoardCardBody };
