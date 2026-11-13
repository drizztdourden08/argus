/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import { useBoards } from '../../../data/boards';
import { resolvePalette } from '../../../palette';
import { BoardIcon } from '../../BoardIcon';
import type { BoardOptionIconProps } from '../BoardSwitcher.type';

const BoardOptionIcon = ({ card }: BoardOptionIconProps) => {
  const boards = useBoards();
  return (
    <Box as="span" className="board-switcher__icon" data-palette={resolvePalette(card.key, card.palette, boards)}>
      <BoardIcon boardKey={card.key} cardIcon={card.icon} size="sm" />
    </Box>
  );
};

export { BoardOptionIcon };
