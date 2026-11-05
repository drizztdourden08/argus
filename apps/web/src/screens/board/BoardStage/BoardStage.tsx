/* @layer renderer-app @kind component */
import { ContentSwap } from '@drizztdourden08/tessera';
import { useBoard, usePortfolio } from '../../../data/react';
import { resolvePalette, useShellPalette } from '../../../palette';
import { BoardPage } from '../BoardPage';
import { useShownBoard } from './behavior/useShownBoard';
import type { BoardStageProps } from './BoardStage.type';

const BoardStage = (props: BoardStageProps) => {
  const { boardKey } = props;
  const { ready } = useBoard(boardKey);
  const { cards } = usePortfolio();
  const shown = useShownBoard(boardKey, ready);
  useShellPalette(resolvePalette(shown, cards.find((card) => card.key === shown)?.palette));
  return (
    <ContentSwap swapKey={shown} effect="fade">
      <BoardPage boardKey={shown} />
    </ContentSwap>
  );
};

export { BoardStage };
