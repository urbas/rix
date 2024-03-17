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

test("'builtins.head' on lists", () => {
  expect(
    getBuiltin("head").apply(new NixList([new NixFloat(1), new NixFloat(2)])),
  ).toStrictEqual(new NixFloat(1));
});

test("'builtins.head' throws when list is empty", () => {
  expect(() => getBuiltin("head").apply(new NixList([]))).toThrow(
    new EvalException("Cannot fetch the first element in an empty list."),
  );
});

test("'builtins.head' on non-lists throws", () => {
  expect(() => getBuiltin("head").apply(new NixFloat(1))).toThrow(
    new EvalException("Cannot apply the 'head' function on 'float'."),
  );
});
