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

test("'builtins.attrValues' on sets", () => {
  expect(
    getBuiltin("attrValues")
      .apply(
        new StrictAttrset(
          new Map<string, NixType>([
            ["b", n.FALSE],
            ["a", n.TRUE],
            ["c", new NixFloat(1)],
          ]),
        ),
      )
      .toJs(),
  ).toStrictEqual([true, false, 1]);
});

test("'builtins.attrValues' on non-sets throws", () => {
  expect(() => getBuiltin("attrValues").apply(n.TRUE)).toThrow(
    new EvalException("Cannot apply the 'attrValues' function on 'bool'."),
  );
});
