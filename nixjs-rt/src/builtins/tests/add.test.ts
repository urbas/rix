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

test("'builtins.add' adds two numbers", () => {
  expect(
    getBuiltin("add").apply(new NixInt(1n)).apply(new NixInt(2n)),
  ).toStrictEqual(new NixInt(3n));
  expect(
    getBuiltin("add").apply(new NixFloat(1)).apply(new NixFloat(2)),
  ).toStrictEqual(new NixFloat(3));
});

test("'builtins.add' throws when trying to add two strings", () => {
  expect(() =>
    getBuiltin("add").apply(new NixString("a")).apply(new NixString("b")),
  ).toThrow(
    new EvalException("value is of type 'string' while a number was expected."),
  );
  expect(() =>
    getBuiltin("add").apply(new NixFloat(1)).apply(new NixString("b")),
  ).toThrow(
    new EvalException("value is of type 'string' while a number was expected."),
  );
});
