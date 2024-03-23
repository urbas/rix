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

test("'builtins.all' on lists", () => {
  expect(
    getBuiltin("all")
      .apply(new Lambda((ctx) => ctx))
      .apply(new NixList([n.TRUE, n.TRUE, n.TRUE])),
  ).toBe(n.TRUE);
  expect(
    getBuiltin("all")
      .apply(new Lambda((ctx) => ctx))
      .apply(new NixList([n.TRUE, n.FALSE, n.TRUE])),
  ).toBe(n.FALSE);
});

test("'builtins.all' on non-function throws", () => {
  expect(() => getBuiltin("all").apply(new NixFloat(1))).toThrow(
    new EvalException(
      "'all' function requires another function, but got 'float' instead.",
    ),
  );
});

test("'builtins.all' on non-list throws", () => {
  expect(() =>
    getBuiltin("all")
      .apply(new Lambda((ctx) => ctx))
      .apply(n.TRUE),
  ).toThrow(new EvalException("Cannot apply the 'all' function on 'bool'."));
});
