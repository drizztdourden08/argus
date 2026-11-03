/* @layer renderer-app @kind component */
import { TableCell, TableRow, Text, TypeMark } from '@drizztdourden08/tessera';
import type { IconName } from '@drizztdourden08/tessera';
import { ITEM_REGISTRY } from '../../../../../data/registry';
import { categoryColor } from '../model';

const TypeRows = () => (
  <>
    {ITEM_REGISTRY.types.filter((type) => type.collection === 'items').map((type) => (
      <TableRow key={type.id}>
        <TableCell><TypeMark icon={type.icon as IconName} label={type.label} color={categoryColor(type.color)} size="xs" /></TableCell>
        <TableCell><Text variant="subtitle">{type.label}</Text></TableCell>
      </TableRow>
    ))}
  </>
);

export { TypeRows };
