/* @layer renderer-app @kind component */
import { Box, Icon, Image } from '@drizztdourden08/tessera';
import { BRAND_FAMILY, BrandMark } from '@drizztdourden08/tessera/brand';
import { useBoards } from '../../data/boards';
import { resolveBoardIcon } from '../../board-icon';
import { BOARD_ICON_PX } from './BoardIcon.constants';
import type { BoardIconProps } from './BoardIcon.type';
import './BoardIcon.css';

const BoardIcon = (props: BoardIconProps) => {
  const { source, boardKey = '', icon, avatar, cardIcon, size = 'md', title } = props;
  useBoards();
  const shown = source ?? resolveBoardIcon(boardKey, icon, avatar, cardIcon).source;
  const px = BOARD_ICON_PX[size];
  const sized = `board-icon board-icon--${size}`;
  if (shown.kind === 'brand') {
    return (
      <Box as="span" className={sized}>
        <BrandMark app={shown.app} size={size} variant="app-icon" title={title ?? BRAND_FAMILY[shown.app].name} />
      </Box>
    );
  }
  if (shown.kind === 'image') {
    return <Image className={`${sized} board-icon--image`} src={shown.src} alt={title ?? ''} width={px} height={px} fit="contain" />;
  }
  return (
    <Box as="span" className={`${sized} board-icon--tile`} role={title ? 'img' : undefined} aria-label={title}>
      <Icon name={shown.name} size={Math.round(px * 0.6)} aria-hidden />
    </Box>
  );
};

export { BoardIcon };
