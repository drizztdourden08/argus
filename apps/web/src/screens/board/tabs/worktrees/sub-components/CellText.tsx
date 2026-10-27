/* @layer renderer-app @kind component */
import { Box, Text } from '@drizztdourden08/tessera';
import type { CellTextProps } from '../worktrees-props.type';
import './CellText.css';

const CellText = (props: CellTextProps) => {
  const { fill = false, tail = 0, children, ...text } = props;
  if (tail > 0 && typeof children === 'string' && children.length > tail) {
    return (
      <Text {...text} className="cell-text cell-text--middle">
        <Box as="span" className="cell-text__head">{children.slice(0, -tail)}</Box>
        <Box as="span" className="cell-text__tail">{children.slice(-tail)}</Box>
      </Text>
    );
  }
  return <Text {...text} className={fill ? 'cell-text cell-text--fill' : 'cell-text'}>{children}</Text>;
};

export { CellText };
