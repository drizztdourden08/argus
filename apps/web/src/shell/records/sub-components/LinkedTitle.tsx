/* @layer renderer-app @kind component */
import { Box, Text } from '@drizztdourden08/tessera';
import type { KeyboardEvent, MouseEvent } from 'react';
import type { LinkedTitleProps } from '../records.type';
import { LinkedText } from './LinkedText';

const OPEN_KEYS: readonly string[] = ['Enter', ' '];

const LinkedTitle = (props: LinkedTitleProps) => {
  const { text, board, onOpen, weight, clamp, label } = props;
  const click = (event: MouseEvent) => {
    if (event.target instanceof Element && event.target.closest('a') !== null) return;
    onOpen();
  };
  const press = (event: KeyboardEvent) => {
    if (event.target === event.currentTarget && OPEN_KEYS.includes(event.key)) onOpen();
  };
  const button = label === undefined ? {} : { role: 'button', tabIndex: 0, 'aria-label': label, onKeyDown: press };
  return (
    <Box onClick={click} data-linked-title="" {...button}>
      <Text weight={weight} clamp={clamp}>
        <LinkedText text={text} board={board} />
      </Text>
    </Box>
  );
};

export { LinkedTitle };
