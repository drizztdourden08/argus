/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { BoardMarkdown } from '../../..';
import { enumOptions } from '../model/enum-options';
import type { FieldBodyProps } from './fields.type';
import { FieldLines } from './FieldLines';
import { RefTags } from './RefTags';

const FieldBody = (props: FieldBodyProps) => {
  const { spec, text, label } = props;
  if (spec.content === 'rich') return <BoardMarkdown>{text}</BoardMarkdown>;
  if (spec.content === 'ref' || spec.content === 'refs') return <RefTags ids={text.split(/[\s,]+/).filter(Boolean)} />;
  if (spec.content === 'enum') return <Text>{enumOptions(spec).find((option) => option.value === text)?.label ?? text}</Text>;
  if (spec.list !== undefined || text.includes('\n')) return <FieldLines label={label} lines={text.split('\n')} />;
  return <Text>{text}</Text>;
};

export { FieldBody };
