/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import type { MouseEvent } from 'react';
import type { ItemGroupHeaderProps } from '../item-groups.type';
import { ItemRecordRow } from './ItemRecordRow';

const keepOpen = (event: MouseEvent): void => event.preventDefault();

const ItemGroupHeader = (props: ItemGroupHeaderProps) => (
  <Box onClick={keepOpen} data-item-group={props.item.id}>
    <ItemRecordRow {...props} />
  </Box>
);

export { ItemGroupHeader };
