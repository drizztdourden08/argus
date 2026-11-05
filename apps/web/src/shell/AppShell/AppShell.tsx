/* @layer renderer-app @kind component */
import { BackdropLayer } from '@drizztdourden08/tessera';
import { Suspense } from 'react';
import { Outlet } from 'react-router';
import { MeshBackdrop } from '../backdrop';
import { LoadingFallback } from '../LoadingFallback';
import { PageFrame } from '../PageFrame';
import { ShellBar } from '../ShellBar';
import { usePreloadPages } from './behavior/usePreloadPages';
import { useShellMode } from './behavior/useShellMode';
import './AppShell.css';

const AppShell = () => {
  const { mode, boardKey, boardsAvailable, select } = useShellMode();
  usePreloadPages();
  return (
    <div className="app-shell">
      <BackdropLayer className="app-shell__backdrop" backdrop={{ kind: 'node', node: <MeshBackdrop /> }} />
      <ShellBar mode={mode} boardKey={boardKey} boardsAvailable={boardsAvailable} onMode={select} />
      <PageFrame>
        <Suspense fallback={<LoadingFallback />}>
          <Outlet />
        </Suspense>
      </PageFrame>
    </div>
  );
};

export { AppShell };
