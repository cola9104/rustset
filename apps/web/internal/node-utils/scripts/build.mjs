import { spawnSync } from 'node:child_process';

// Scripts run with the workspace's local node_modules/.bin on PATH, so the
// tools can be invoked directly. Under bun, npm_execpath points at the bun
// runtime; the legacy pnpm execpath fallback is kept for pnpm compatibility.
const baseArgs =
  process.env.npm_execpath && process.env.npm_execpath.endsWith('.cjs')
    ? [process.execPath, process.env.npm_execpath, 'exec']
    : [];

const steps = [
  ['tsdown', '--no-dts'],
  [
    'tsc',
    '-p',
    'tsconfig.build.json',
    '--emitDeclarationOnly',
    '--declaration',
    '--outDir',
    'dist',
  ],
];

for (const args of steps) {
  const command = baseArgs[0] ?? args[0];
  const commandArgs = baseArgs.length > 0 ? [...baseArgs.slice(1), ...args] : args.slice(1);
  const result = spawnSync(command, commandArgs, {
    shell: true,
    stdio: 'inherit',
  });

  if (result.error) {
    throw result.error;
  }

  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}
