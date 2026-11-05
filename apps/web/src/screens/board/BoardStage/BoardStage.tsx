/* @layer renderer-app @kind component */
import { useBoard, usePortfolio } from '../../../data/react';
import { resolvePalette, useShellPalette } from '../../../palette';
import { BoardPage } from '../BoardPage';
import { useBoardSwitchPhase } from './behavior/useBoardSwitchPhase';
import type { BoardStageProps } from './BoardStage.type';
import './BoardStage.css';

const BoardStage = (props: BoardStageProps) => {
  const { boardKey } = props;
  const { ready } = useBoard(boardKey);
  const { cards } = usePortfolio();
  const { shown, phase } = useBoardSwitchPhase(boardKey, ready);
  useShellPalette(resolvePalette(shown, cards.find((card) => card.key === shown)?.palette));
  return (
    <div className="board-stage" data-phase={phase}>
      <BoardPage key={shown} boardKey={shown} />
    </div>
  );
};

export { BoardStage };
