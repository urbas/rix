import { beforeEach, expect, test } from "@jest/globals";
import n, {
  Attrset,
  attrset,
  AttrsetBody,
  EMPTY_ATTRSET,
  EvalCtx,
  EvalException,
  Lambda,
  Lazy,
  NixFloat,
  NixInt,
  NixList,
  NixString,
  NixType,
  Path,
  StrictAttrset,
} from "../../lib";
import { evalCtx, getBuiltin } from "../../testUtils";

test("'builtins.any' on lists", () => {
  expect(
    getBuiltin("any")
      .apply(new Lambda((ctx) => ctx))
      .apply(new NixList([n.FALSE, n.FALSE, n.TRUE])),
  ).toBe(n.TRUE);
  expect(
    getBuiltin("any")
      .apply(new Lambda((ctx) => ctx))
      .apply(new NixList([n.FALSE, n.FALSE, n.FALSE])),
  ).toBe(n.FALSE);
});

test("'builtins.any' on non-function throws", () => {
  expect(() => getBuiltin("any").apply(new NixFloat(1))).toThrow(
    new EvalException(
      "'any' function requires another function, but got 'float' instead.",
    ),
  );
});

test("'builtins.any' on non-list throws", () => {
  expect(() =>
    getBuiltin("any")
      .apply(new Lambda((ctx) => ctx))
      .apply(n.TRUE),
  ).toThrow(new EvalException("Cannot apply the 'any' function on 'bool'."));
});
