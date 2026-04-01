import fs from 'node:fs';
import path from 'node:path';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

const appRoot = process.cwd();
const extraAllow = path.resolve(appRoot, '../../../../apps/lwe');
const fsAllow = [appRoot];

if (fs.existsSync(extraAllow)) {
  fsAllow.push(extraAllow);
}

export default defineConfig({
  plugins: [sveltekit()],
  server: {
    fs: {
      allow: fsAllow
    }
  }
});
