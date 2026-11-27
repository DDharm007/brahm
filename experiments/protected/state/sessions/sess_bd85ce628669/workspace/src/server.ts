import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createCommerceApp } from './app.js';
import { loadConfig } from './config/env.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const projectRoot = path.resolve(__dirname, '..');

const config = loadConfig();
const { app } = createCommerceApp(config, projectRoot);

app.listen(config.port, () => {
  console.log(`brahm-commerce-api listening on http://127.0.0.1:${config.port}`);
});
