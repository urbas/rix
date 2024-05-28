import { NixBool, NixNull } from "./lib";

export function isAbsolutePath(path: string): boolean {
  return path.startsWith("/");
}

export function joinPaths(abs_base: string, path: string): string {
  return `${abs_base}/${path}`;
}

export function normalizePath(path: string): string {
  let segments = path.split("/");
  let normalizedSegments: string[] = [];
  for (const segment of segments) {
    switch (segment) {
      case "":
        break;
      case ".":
        break;
      case "..":
        normalizedSegments.pop();
        break;
      default:
        normalizedSegments.push(segment);
        break;
    }
  }
  return (isAbsolutePath(path) ? "/" : "") + normalizedSegments.join("/");
}

export function dirOf(path: string) {
  // Return everything before the final slash
  const lastSlash = path.lastIndexOf("/");
  return path.substring(0, lastSlash);
}

// These are types that there is only 1 kind of each. This means we can re-use
// the same allocation, and we can avoid the cost of creating a new object.
export const NULL = new NixNull();
export const TRUE = new NixBool(true);
export const FALSE = new NixBool(false);

// For creating a bool without allocating a new object.
export function nixBoolFromJs(value: boolean): NixBool {
  return value ? TRUE : FALSE;
}
