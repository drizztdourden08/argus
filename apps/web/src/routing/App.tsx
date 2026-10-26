/* @layer renderer-app @kind component */
import { TesseraProvider } from '@drizztdourden08/tessera';
import { BrowserRouter } from 'react-router';
import { DataProvider } from '../data/react';
import { AppRoutes } from './AppRoutes';

const App = () => (
  <TesseraProvider overrides={{}}>
    <DataProvider>
      <BrowserRouter>
        <AppRoutes />
      </BrowserRouter>
    </DataProvider>
  </TesseraProvider>
);

export { App };
