import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
export function sha256(path: string) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}
export function assertStaticElf(data: Buffer) {
  if (
    data.length < 64 ||
    !data.subarray(0, 4).equals(Buffer.from([127, 69, 76, 70])) ||
    data[4] !== 2 ||
    data[5] !== 1 ||
    data.readUInt16LE(18) !== 62
  )
    throw new Error("Expected a little-endian x86_64 ELF64 artifact");
  const offset = Number(data.readBigUInt64LE(32));
  const size = data.readUInt16LE(54),
    count = data.readUInt16LE(56);
  if (
    size < 56 ||
    !Number.isSafeInteger(offset) ||
    offset + size * count > data.length
  )
    throw new Error("Invalid ELF program headers");
  for (let i = 0; i < count; i++) {
    const header = offset + size * i,
      type = data.readUInt32LE(header);
    if (type === 3) throw new Error("Unexpected ELF interpreter");
    if (type !== 2) continue;
    const start = Number(data.readBigUInt64LE(header + 8)),
      length = Number(data.readBigUInt64LE(header + 32));
    if (
      !Number.isSafeInteger(start) ||
      !Number.isSafeInteger(length) ||
      start + length > data.length
    )
      throw new Error("Invalid ELF dynamic segment");
    for (let at = start; at + 16 <= start + length; at += 16) {
      const tag = data.readBigInt64LE(at);
      if (tag === 0n) break;
      if (tag === 1n)
        throw new Error("Unexpected ELF shared library dependency");
    }
  }
}
