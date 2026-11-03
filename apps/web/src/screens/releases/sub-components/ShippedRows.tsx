/* @layer renderer-app @kind component */
import { Disclosure, Flex } from '@drizztdourden08/tessera';
import { RecordLink } from '../../../shell';
import { ItemRecordRow } from '../../board/item-groups';
import { shippedOf } from '../model';
import type { ShippedRowsProps } from '../releases.type';
import { ShippedPrRow } from './ShippedPrRow';

const ShippedRows = (props: ShippedRowsProps) => {
  const { version, project, source } = props;
  const shipped = shippedOf(version, source);
  const count = shipped.items.length + shipped.unknown.length;
  if (count === 0 && shipped.prs.length === 0) return null;
  return (
    <Disclosure size="sm" summary={`Ships ${count} ${count === 1 ? 'item' : 'items'} and ${shipped.prs.length} ${shipped.prs.length === 1 ? 'pull request' : 'pull requests'}`}>
      <Flex direction="column" gap="xs" role="list" aria-label={`Shipped in ${version.version}`}>
        {shipped.items.map((item) => <ItemRecordRow key={item.id} item={item} boardKey={project} threads={[]} />)}
        {shipped.unknown.length > 0 && (
          <Flex gap="sm" wrap>
            {shipped.unknown.map((id) => <RecordLink key={id} id={id} board={project} />)}
          </Flex>
        )}
        {shipped.prs.map(({ pr, local }) => <ShippedPrRow key={pr.commit} boardKey={project} pr={pr} local={local} />)}
      </Flex>
    </Disclosure>
  );
};

export { ShippedRows };
