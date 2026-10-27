/* @layer renderer-app @kind component */
import { Box, Pressable } from '@drizztdourden08/tessera';
import { useNavigate } from 'react-router';
import type { AgentItemProps } from '../row-props.type';
import { AgentCell } from './AgentCell';

const AgentItem = (props: AgentItemProps) => {
  const { item } = props;
  const navigate = useNavigate();
  return (
    <AgentCell name="item">
      {item ? (
        <Pressable className="agents-ref" title={`Open ${item}`} onClick={(event) => {
          event.stopPropagation();
          void navigate({ hash: item });
        }}
        >
          {item}
        </Pressable>
      ) : <Box as="span" className="agents-na">-</Box>}
    </AgentCell>
  );
};

export { AgentItem };
