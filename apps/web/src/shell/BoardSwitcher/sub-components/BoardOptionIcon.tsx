/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import { resolvePalette } from '../../../palette';
import { BoardIcon } from '../../BoardIcon';
import type { BoardOptionIconProps } from '../BoardSwitcher.type';

const BoardOptionIcon = ({ card }: BoardOptionIconProps) => (
  <Box as="span" className="board-switcher__icon" data-palette={resolvePalette(card.key, card.palette)}>
    <BoardIcon boardKey={card.key} cardIcon={card.icon} size="sm" />
  </Box>
);

export { BoardOptionIcon };
