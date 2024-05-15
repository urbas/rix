import type { NixType, EvalCtx } from "./lib";

declare global {
  /**
   * Import a Nix module from the given path. The path is absolute.
   * Returns a transpiled version of the module, executed, with a function
   * that takes an EvalCtx and returns the module's value.
   */
  var importNixModule: (path: string) => (ctx: EvalCtx) => NixType;

  /**
   * Log the string provided, purely for debugging purposes.
   */
  var debugLog: (log: string) => void;
}
