/* @layer renderer-app @kind component */
import { Flex, Link } from '@drizztdourden08/tessera';
import { useNavigate } from 'react-router';
import { itemCounts } from '../model';
import type { ItemLinksProps } from '../media.type';

const ItemLinks = ({ boardKey, shots }: ItemLinksProps) => {
  const navigate = useNavigate();
  return (
    <Flex gap="sm" wrap role="group" aria-label="Items with screenshots">
      {itemCounts(shots).map((row) => (
        <Link key={row.id} href={`/p/${encodeURIComponent(boardKey)}#${row.id}`} navigate={navigate} title={row.title}>{`${row.id} (${row.count})`}</Link>
      ))}
    </Flex>
  );
};

export { ItemLinks };
