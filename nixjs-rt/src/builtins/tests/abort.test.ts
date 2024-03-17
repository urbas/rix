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

test("'builtins.abort' throws with the given message", () => {
  expect(() => getBuiltin("abort").apply(new NixString("foo"))).toThrow(
    new EvalException(
      "Evaluation aborted with the following error message: 'foo'",
    ),
  );
  // `abort` is special, since it's available directly on the global scope
  expect(() => evalCtx().lookup("abort").apply(new NixString("foo"))).toThrow(
    new EvalException(
      "Evaluation aborted with the following error message: 'foo'",
    ),
  );
});

test("'builtins.abort' on a non-string throws during coercion", () => {
  expect(() => getBuiltin("abort").apply(new NixFloat(1))).toThrow(
    new EvalException("Value is 'float' but a string was expected."),
  );
});
