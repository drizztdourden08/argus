/* @layer renderer-app @kind component */
import { ListItemRow, Tag, Text } from '@drizztdourden08/tessera';
import { useProjectName } from '../../projects';
import { ShortAge } from '../../sub-components/ShortAge';
import { GOOD_LOOK, INFO_LOOK, KIND_WORD, WARN_LOOK } from '../dashboards.constants';
import type { SentRowProps } from '../dashboards.type';
import { sentTarget } from '../model';

const SentRow = (props: SentRowProps) => {
  const { entry } = props;
  const nameOf = useProjectName();
  const kind = entry.kind ?? '';
  const target = sentTarget(entry.project, nameOf);
  const from = entry.author === 'human' ? 'you' : 'Overlord';
  return (
    <ListItemRow
      icon={<Tag {...(kind === 'decision' ? INFO_LOOK : GOOD_LOOK)}>{KIND_WORD[kind] ?? kind}</Tag>}
      name={entry.text ?? ''}
      meta={`${entry.id} to ${target}`}
      columns={[
        { primary: <Tag {...(entry.status === 'delivered' ? GOOD_LOOK : WARN_LOOK)}>{entry.status ?? 'sent'}</Tag> },
        { primary: <Text>{from}</Text>, secondary: entry.createdAt ? <ShortAge value={entry.createdAt} /> : '', align: 'end' },
      ]}
    />
  );
};

export { SentRow };
