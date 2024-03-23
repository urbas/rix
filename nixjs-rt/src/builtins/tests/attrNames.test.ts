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

test("'builtins.attrNames' on sets", () => {
  expect(
    getBuiltin("attrNames")
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
  ).toStrictEqual(["a", "b", "c"]);
});

test("'builtins.attrNames' on non-sets throws", () => {
  expect(() => getBuiltin("attrNames").apply(n.TRUE)).toThrow(
    new EvalException("Cannot apply the 'attrNames' function on 'bool'."),
  );
});
