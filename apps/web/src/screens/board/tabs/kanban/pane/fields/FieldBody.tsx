/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { BoardMarkdown } from '../../../../../../shell';
import { enumOptions } from '../model/enum-options';
import type { FieldBodyProps } from '../pane.type';
import { RefTags } from './RefTags';

const FieldBody = (props: FieldBodyProps) => {
  const { spec, text } = props;
  if (spec.content === 'rich') return <BoardMarkdown>{text}</BoardMarkdown>;
  if (spec.content === 'ref' || spec.content === 'refs') return <RefTags ids={text.split(/[\s,]+/).filter(Boolean)} />;
  if (spec.content === 'enum') return <Text>{enumOptions(spec).find((option) => option.value === text)?.label ?? text}</Text>;
  return <Text>{text}</Text>;
};

export { FieldBody };
