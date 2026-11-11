/* @layer renderer-app @kind component */
import './ReviewTree.css';
import { Box } from '@drizztdourden08/tessera';
import { Fragment } from 'react';
import type { ReviewTreeProps } from '../review-tree-props.type';
import { TreeGutter } from './TreeGutter';
import { TreePanel } from './TreePanel';

const ReviewTree = (props: ReviewTreeProps) => {
  const { accounts, rows, hovered, env } = props;
  return (
    <Box className="review-tree">
      {accounts.map((account, index) => {
        const next = accounts[index + 1];
        return (
          <Fragment key={account.key}>
            <TreePanel account={account} rows={rows} hovered={hovered} env={env} />
            {next && <TreeGutter pair={{ left: account.key, right: next.key }} rows={rows} hovered={hovered} env={env} />}
          </Fragment>
        );
      })}
    </Box>
  );
};

export { ReviewTree };
