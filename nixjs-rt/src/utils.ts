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
