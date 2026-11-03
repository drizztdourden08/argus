/* @layer renderer-app @kind component */
import { Flex, Icon, Pressable, Text } from '@drizztdourden08/tessera';
import { Fragment } from 'react';
import { itemCode } from '../behavior/item-code';
import { ItemTypeMark } from '../sub-components/ItemTypeMark';
import { crumbsOf } from './model/crumbs-of';
import type { PaneCrumbsProps } from './pane.type';

const PaneCrumbs = (props: PaneCrumbsProps) => {
  const { item, items, onOpen } = props;
  const crumbs = crumbsOf(item, items);
  if (crumbs.length === 0) return null;
  return (
    <Flex gap="xs" align="center" wrap role="navigation" aria-label="Parents">
      {crumbs.map((crumb, index) => (
        <Fragment key={crumb.id}>
          {index > 0 && <Icon name="chevron-right" />}
          <Pressable aria-label={`Open ${itemCode(crumb)}: ${crumb.title}`} onClick={() => onOpen(itemCode(crumb))}>
            <Flex gap="xs" align="center">
              <ItemTypeMark item={crumb} />
              <Text as="span" variant="caption" mono>{itemCode(crumb)}</Text>
              <Text as="span" variant="caption" smartTimes>{crumb.title}</Text>
            </Flex>
          </Pressable>
        </Fragment>
      ))}
    </Flex>
  );
};

export { PaneCrumbs };
