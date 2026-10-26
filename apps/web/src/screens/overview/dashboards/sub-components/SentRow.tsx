/* @layer renderer-app @kind component */
import { ListItemRow, Tag, Text } from '@drizztdourden08/tessera';
import { formatAgo } from '../../format';
import { useProjectName } from '../../projects';
import { GOOD_LOOK, INFO_LOOK, KIND_WORD, WARN_LOOK } from '../dashboards.constants';
import type { SentRowProps } from '../dashboards.type';

const SentRow = (props: SentRowProps) => {
  const { entry, now } = props;
  const nameOf = useProjectName();
  const kind = entry.kind ?? '';
  const target = entry.project === 'all' ? 'all projects' : nameOf(entry.project ?? '');
  const from = entry.author === 'human' ? 'you' : 'Overlord';
  return (
    <ListItemRow
      icon={<Tag {...(kind === 'decision' ? INFO_LOOK : GOOD_LOOK)}>{KIND_WORD[kind] ?? kind}</Tag>}
      name={entry.text ?? ''}
      meta={`${entry.id} to ${target}`}
      columns={[
        { primary: <Tag {...(entry.status === 'delivered' ? GOOD_LOOK : WARN_LOOK)}>{entry.status ?? 'sent'}</Tag> },
        { primary: <Text>{from}</Text>, secondary: `${formatAgo(entry.createdAt, now)} ago`, align: 'end' },
      ]}
    />
  );
};

export { SentRow };
