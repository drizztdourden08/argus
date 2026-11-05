/* @layer renderer-app @kind component */
import { Button, Flex } from '@drizztdourden08/tessera';
import { clearPatch } from '../behavior/clear-patch';
import { fieldIsSet } from '../behavior/field-is-set';
import type { TabFieldControlsProps } from '../BoardTabsRow.type';
import { TabFieldControl } from './TabFieldControl';

const TabFieldControls = <F,>({ fields, filters, patch }: TabFieldControlsProps<F>) => (
  <Flex inline gap="sm" align="center" wrap>
    {fields.map((field) => <TabFieldControl key={field.path} field={field} filters={filters} patch={patch} />)}
    {fields.some((field) => fieldIsSet(field, filters)) && (
      <Button size="sm" variant="ghost" onClick={() => patch(clearPatch(fields))}>Clear</Button>
    )}
  </Flex>
);

export { TabFieldControls };
