import * as fs from 'fs';
import * as path from 'path';

const PID_FILE = path.join(__dirname, '.test-pids.json');

function killProcess(pid: number | undefined) {
  if (!pid) return;
  try {
    process.kill(-pid, 'SIGTERM');
  } catch {
    try {
      process.kill(pid, 'SIGTERM');
    } catch {
      // already dead
    }
  }
}

async function globalTeardown() {
  console.log('Tearing down test infrastructure...');
  try {
    const pids = JSON.parse(fs.readFileSync(PID_FILE, 'utf-8'));
    killProcess(pids.trunk);
    killProcess(pids.spacetime);
    fs.unlinkSync(PID_FILE);
  } catch {
    // PID file may not exist if setup failed
  }
}

export default globalTeardown;
