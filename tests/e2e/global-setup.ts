import { spawn, execSync, ChildProcess } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';

const PID_FILE = path.join(__dirname, '.test-pids.json');
const PROJECT_ROOT = path.resolve(__dirname, '../..');

async function waitForUrl(url: string, timeoutMs = 60_000): Promise<void> {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      const res = await fetch(url);
      if (res.ok) return;
    } catch {
      // not ready yet
    }
    await new Promise(r => setTimeout(r, 1000));
  }
  throw new Error(`Timed out waiting for ${url}`);
}

async function globalSetup() {
  console.log('Starting SpacetimeDB...');
  const spacetime = spawn('spacetime', ['start'], {
    cwd: PROJECT_ROOT,
    stdio: 'pipe',
    detached: true,
  });
  spacetime.unref();

  // Wait for SpacetimeDB to be ready
  await waitForUrl('http://localhost:3000/database/ping', 30_000).catch(() => {
    return new Promise(r => setTimeout(r, 5000));
  });

  console.log('Publishing server module...');
  execSync('spacetime publish -s local collaborate -p crates/server -c=always --yes', {
    cwd: PROJECT_ROOT,
    stdio: 'inherit',
  });

  console.log('Starting Trunk dev server...');
  const trunk = spawn('trunk', ['serve', '--port', '8090'], {
    cwd: path.join(PROJECT_ROOT, 'crates/app'),
    stdio: 'pipe',
    detached: true,
  });
  trunk.unref();

  await waitForUrl('http://localhost:8090', 60_000);

  fs.writeFileSync(PID_FILE, JSON.stringify({
    spacetime: spacetime.pid,
    trunk: trunk.pid,
  }));

  console.log('Test infrastructure ready.');
}

export default globalSetup;
