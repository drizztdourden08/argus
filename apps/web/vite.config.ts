/* @layer root-config @kind config */
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

const SERVER = 'http://127.0.0.1:47615';

const REACT_PACKAGES = /node_modules[\\/](react|react-dom|react-router|scheduler)[\\/]/;

export default defineConfig({
  plugins: [react()],
  resolve: { dedupe: ['react', 'react-dom'] },
  build: {
    rollupOptions: {
      output: {
        manualChunks: (id) => (REACT_PACKAGES.test(id) ? 'react' : undefined),
      },
    },
  },
  server: {
    proxy: {
      '/api': { target: SERVER, changeOrigin: true },
      '/mcp': { target: SERVER, changeOrigin: true },
      '/files': { target: SERVER, changeOrigin: true },
    },
  },
});
