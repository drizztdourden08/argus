/* @layer renderer-app @kind component */
import { useBoards } from '../../../data/boards';
import { useBoard, usePortfolio } from '../../../data/react';
import { resolvePalette, useShellPalette } from '../../../palette';
import { BoardPage } from '../BoardPage';
import { useShownBoard } from './behavior/useShownBoard';
import type { BoardStageProps } from './BoardStage.type';

const BoardStage = (props: BoardStageProps) => {
  const { boardKey } = props;
  const { ready } = useBoard(boardKey);
  const { cards } = usePortfolio();
  const boards = useBoards();
  const shown = useShownBoard(boardKey, ready);
  useShellPalette(resolvePalette(shown, cards.find((card) => card.key === shown)?.palette, boards));
  return <BoardPage boardKey={shown} />;
};

export { BoardStage };
