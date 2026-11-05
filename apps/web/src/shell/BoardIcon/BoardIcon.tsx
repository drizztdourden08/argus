/* @layer renderer-app @kind component */
import { Icon } from '@drizztdourden08/tessera';
import { BRAND_FAMILY, BrandMark } from '@drizztdourden08/tessera/brand';
import type { CSSProperties } from 'react';
import { resolveBoardIcon } from '../../board-icon';
import { BOARD_ICON_PX } from './BoardIcon.constants';
import type { BoardIconProps } from './BoardIcon.type';
import './BoardIcon.css';

const BoardIcon = (props: BoardIconProps) => {
  const { source, boardKey = '', icon, avatar, cardIcon, size = 'md', title } = props;
  const shown = source ?? resolveBoardIcon(boardKey, icon, avatar, cardIcon).source;
  const px = BOARD_ICON_PX[size];
  const style = { '--board-icon-size': `${px}px` } as CSSProperties;
  if (shown.kind === 'brand') {
    return (
      <span className="board-icon" style={style}>
        <BrandMark app={shown.app} size={size} variant="app-icon" title={title ?? BRAND_FAMILY[shown.app].name} />
      </span>
    );
  }
  if (shown.kind === 'image') {
    return <img className="board-icon board-icon--image" style={style} src={shown.src} alt={title ?? ''} />;
  }
  return (
    <span className="board-icon board-icon--tile" style={style} role={title ? 'img' : undefined} aria-label={title}>
      <Icon name={shown.name} size={Math.round(px * 0.6)} aria-hidden />
    </span>
  );
};

export { BoardIcon };
