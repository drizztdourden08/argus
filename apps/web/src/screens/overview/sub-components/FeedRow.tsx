/* @layer renderer-app @kind component */
import { Icon, TableCell, TableRow, Text } from '@drizztdourden08/tessera';
import type { FeedRowProps } from '../overview.type';
import { ShortAge } from './ShortAge';

const FeedRow = (props: FeedRowProps) => {
  const { icon, tone, title, at, children } = props;
  return (
    <TableRow>
      <TableCell>
        <Text tone={tone} title={title}><Icon name={icon} size={14} /></Text>
      </TableCell>
      <TableCell>
        <Text variant="caption" numeric title={at ?? ''}>{at ? <><ShortAge value={at} />{' ago'}</> : ''}</Text>
      </TableCell>
      <TableCell>{children}</TableCell>
    </TableRow>
  );
};

export { FeedRow };
