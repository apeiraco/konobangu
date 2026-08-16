import { constants } from "node:os";
import { resolve } from "node:path";
import spawn from "cross-spawn";

export const root = resolve(import.meta.dirname, "../..");
export const executable = (name: string) =>
  process.platform === "win32" ? `${name}.exe` : name;
export class CommandError extends Error {
  code: number;
  constructor(code: number, command: string) {
    super(`${command} failed (${code})`);
    this.code = code;
  }
}
export function run(
  command: string,
  args: string[] = [],
  options: {
    env?: NodeJS.ProcessEnv;
    capture?: boolean;
    check?: boolean;
    timeout?: number;
    cwd?: string;
  } = {},
) {
  const result = spawn.sync(command, args, {
    cwd: options.cwd ?? root,
    env: { ...process.env, ...options.env },
    encoding: "utf8",
    stdio: options.capture ? "pipe" : "inherit",
    // Cargo metadata for the workspace exceeds Node's 1 MiB default.
    maxBuffer: 32 * 1024 * 1024,
    timeout: options.timeout,
  });
  if (result.error)
    throw new Error(`${command}: ${result.error.message}`, {
      cause: result.error,
    });
  if (result.signal)
    throw new CommandError(
      128 + (constants.signals[result.signal] ?? 1),
      `${command} terminated by ${result.signal}`,
    );
  if (options.check !== false && result.status !== 0)
    throw new CommandError(result.status ?? 1, command);
  return {
    code: result.status ?? 1,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
  };
}
