/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import type { TodosSectionProps } from '../drawer-props.type';
import { LIMITS, textProblem } from '../model';
import { AddLine } from './AddLine';
import { DrawerSection } from './DrawerSection';
import { TodoRow } from './TodoRow';

const TodosSection = (props: TodosSectionProps) => {
  const { item, todos, ops, acts } = props;
  const open = todos.filter((todo) => !todo.done).length;
  return (
    <DrawerSection title={`To-dos, ${open} open`}>
      {todos.length === 0 && <Text tone="muted">No to-do on this item.</Text>}
      <Flex direction="column" gap="xs">
        {todos.map((todo) => (
          <TodoRow key={todo.action.id} todo={todo} ops={ops} acts={acts} />
        ))}
      </Flex>
      <AddLine
        label={`+ To-do for ${item.thread}`}
        buttonLabel="Add to-do"
        limit={LIMITS.todo}
        problemOf={(text) => textProblem('todo', text)}
        onAdd={(text) => ops.addTodo(item, text)}
      />
    </DrawerSection>
  );
};

export { TodosSection };
