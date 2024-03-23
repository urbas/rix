import { beforeEach } from "@jest/globals";
import { NixType, NixString, EvalCtx, AttrsetBody } from "./lib";
import { execSync } from "child_process";

let _evalCtx: EvalCtx | null = null;
export function evalCtx() {
  if (_evalCtx === null) {
    _evalCtx = new EvalCtx("/test_base");
  }
  return _evalCtx;
}

export function toAttrpath(attrPathStr: string): NixType[] {
  return attrPathStr.split(".").map((val) => new NixString(val) as NixType);
}

export function getBuiltin(builtinName: string): NixType {
  return evalCtx()
    .lookup("builtins")
    .select([new NixString(builtinName)], undefined);
}

export function keyVals(
  ...entries: [attrpathStr: string, value: NixType][]
): AttrsetBody {
  return (_ctx) => entries.map((entry) => [toAttrpath(entry[0]), entry[1]]);
}
