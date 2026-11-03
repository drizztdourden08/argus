/* @layer renderer-app @kind component */
import { EmptyState, Flex, Tree } from '@drizztdourden08/tessera';
import { useMemo, useState } from 'react';
import { useFolds } from './behavior/useFolds';
import { useTreeRoots } from './behavior/useTreeRoots';
import { indexNodes } from './model';
import { TreeAssignee } from './sub-components/TreeAssignee';
import { TreeBadges } from './sub-components/TreeBadges';
import { TreeFoldBar } from './sub-components/TreeFoldBar';
import { TreeLabel } from './sub-components/TreeLabel';
import { TreeLead } from './sub-components/TreeLead';
import { TREE_EMPTY, TREE_LABEL } from './tree.constants';
import type { TreeNode, TreeViewProps } from './tree.type';
import './TreeView.css';

const TreeView = (props: TreeViewProps) => {
  const { boardKey, view, source, threads, onOpen } = props;
  const shown = useMemo(() => view.cards.map((card) => card.id), [view.cards]);
  const roots = useTreeRoots(source, shown);
  const nodes = useMemo(() => indexNodes(roots), [roots]);
  const fold = useFolds(roots);
  const [picked, setPicked] = useState<ReadonlySet<string>>(new Set());
  const open = (node: TreeNode | undefined) => {
    if (node) onOpen(node.open);
  };
  if (roots.length === 0) return <EmptyState message={TREE_EMPTY} />;
  return (
    <Flex direction="column" gap="sm">
      <TreeFoldBar fold={fold} />
      <Tree<TreeNode>
        className="item-tree"
        label={TREE_LABEL}
        items={roots}
        getKey={(node) => node.key}
        getChildren={(node) => node.children}
        getLabel={(node) => `${node.id} ${node.title}`}
        expandedKeys={fold.expanded}
        onExpandedChange={fold.onChange}
        selection={{
          selected: picked,
          mode: 'single',
          onChange: (keys) => {
            setPicked(keys);
            open(nodes.get([...keys][0] ?? ''));
          },
        }}
        onActivate={open}
        renderLead={(node) => <TreeLead node={node} boardKey={boardKey} threads={threads} />}
        renderLabel={(node) => <TreeLabel node={node} boardKey={boardKey} threads={threads} />}
        renderBadges={(node) => <TreeBadges node={node} boardKey={boardKey} threads={threads} />}
        renderTrail={(node) => <TreeAssignee node={node} boardKey={boardKey} threads={threads} />}
        guides
        rowSize="normal"
        trail="always"
      />
    </Flex>
  );
};

export { TreeView };
