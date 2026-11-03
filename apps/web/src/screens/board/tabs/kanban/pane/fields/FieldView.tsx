/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { fieldOf } from '../../../../../../data/registry';
import { BoardMarkdown } from '../../../../../../shell';
import { enumOptions } from '../model/enum-options';
import { fieldText } from '../model/field-text';
import type { FieldProps } from '../pane.type';
import { RefTags } from './RefTags';

const NOT_SET = 'not set';

const FieldView = (props: FieldProps) => {
  const { name, item, open } = props;
  const spec = fieldOf(name);
  const value = item.fields?.[name];
  if (spec?.content === 'records') return <Text>{`${Array.isArray(value) ? value.length : 0} recorded`}</Text>;
  const text = fieldText(name, value);
  if (!spec || text === '') return <Text tone="muted" italic>{NOT_SET}</Text>;
  if (spec.content === 'rich') return <BoardMarkdown>{text}</BoardMarkdown>;
  if (spec.content === 'ref' || spec.content === 'refs') return <RefTags ids={text.split(/[\s,]+/).filter(Boolean)} open={open} />;
  if (spec.content === 'enum') return <Text>{enumOptions(spec).find((option) => option.value === text)?.label ?? text}</Text>;
  return <Text>{text}</Text>;
};

export { FieldView };
