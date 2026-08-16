import { cpSync, mkdirSync } from "node:fs";
import { createConnection } from "node:net";
import { join } from "node:path";
import { setTimeout } from "node:timers/promises";
import { root } from "../lib/process.mts";
export function copyWebui() {
  mkdirSync(join(root, "apps/recorder/webui"), { recursive: true });
  cpSync(join(root, "apps/webui/dist"), join(root, "apps/recorder/webui"), {
    recursive: true,
  });
}
export async function waitRecorder() {
  const deadline = performance.now() + 120000;
  while (performance.now() < deadline) {
    let ready: boolean;
    {
      using attempt = new DisposableStack();
      const socket = attempt.adopt(
        createConnection({ host: "127.0.0.1", port: 5001 }),
        (socket) => socket.destroy(),
      );
      ready = await new Promise<boolean>((resolve) => {
        socket.once("connect", () => resolve(true));
        socket.once("error", () => resolve(false));
        socket.setTimeout(1000, () => resolve(false));
      });
    }
    if (ready) return;
    await setTimeout(500);
  }
  throw new Error("Recorder did not become available within 120 seconds");
}
