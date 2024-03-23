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
} from "./lib";
import { evalCtx, keyVals, toAttrpath } from "./testUtils";

// Apply:
test("calling a lambda should return its value", () => {
  expect(new Lambda((_) => new NixInt(1n)).apply(EMPTY_ATTRSET)).toStrictEqual(
    new NixInt(1n),
  );
});

test("calling something that isn't a lambda should throw", () => {
  expect(() => new NixInt(1n).apply(EMPTY_ATTRSET)).toThrow(
    new EvalException(
      "Attempt to call something which is not a function but is 'int'.",
    ),
  );
});

// Arithmetic:
test("unary '-' operator on integers", () => {
  expect(new NixInt(1n).neg()).toStrictEqual(new NixInt(-1n));
});

test("unary '-' operator on floats", () => {
  expect(new NixFloat(2.5).neg()).toStrictEqual(new NixFloat(-2.5));
});

test("unary '-' operator on non-numbers", () => {
  expect(() => new NixString("a").neg()).toThrow(n.EvalException);
});

test("'+' operator on integers", () => {
  expect((new NixInt(1n).add(new NixInt(2n)) as NixInt).number).toBe(3);
  expect(
    (
      new NixInt(4611686018427387904n).add(
        new NixInt(4611686018427387904n),
      ) as NixInt
    ).int64,
  ).toBe(-9223372036854775808n);
});

test("'+' operator on floats", () => {
  expect(new NixFloat(1.0).add(new NixFloat(2.0)).toJs()).toBe(3);
});

test("'+' operator on mixed integers and floats", () => {
  expect(new NixInt(1n).add(new NixFloat(2.0)).toJs()).toBe(3.0);
  expect(new NixFloat(2.0).add(new NixInt(1n)).toJs()).toBe(3.0);
});

test("'+' operator on mixed numbers and non-numbers", () => {
  expect(() => new NixString("a").add(new NixInt(1n))).toThrow(n.EvalException);
  expect(() => new NixInt(1n).add(new NixString("a"))).toThrow(n.EvalException);
  expect(() => new NixFloat(1).add(new NixString("a"))).toThrow(
    n.EvalException,
  );
  expect(() => new NixString("a").add(new NixFloat(1))).toThrow(
    n.EvalException,
  );
});

test("'+' operator on strings", () => {
  expect(new NixString("a").add(new NixString("b")).toJs()).toBe("ab");
});

test("'+' operator on paths and strings", () => {
  expect(new Path("/").add(new NixString("b"))).toStrictEqual(new Path("/b"));
  expect(new Path("/a").add(new NixString("b"))).toStrictEqual(new Path("/ab"));
  expect(new Path("/").add(new NixString("/"))).toStrictEqual(new Path("/"));
  expect(new Path("/").add(new NixString("."))).toStrictEqual(new Path("/"));
  expect(new Path("/a").add(new NixString("."))).toStrictEqual(new Path("/a."));
  expect(new Path("/").add(new NixString("./a"))).toStrictEqual(new Path("/a"));
});

test("'+' operator on paths", () => {
  expect(new Path("/").add(new Path("/a"))).toStrictEqual(new Path("/a"));
  expect(
    n.toPath(evalCtx(), "./a").add(n.toPath(evalCtx(), "./b")),
  ).toStrictEqual(new Path("/test_base/a/test_base/b"));
});

test("'-' operator on integers", () => {
  const result = new NixInt(1n).sub(new NixInt(2n)) as NixInt;
  expect(result.number).toBe(-1);
});

test("'-' operator on floats", () => {
  expect(new NixFloat(1).sub(new NixFloat(2)).toJs()).toBe(-1);
});

test("'-' operator on mixed integers and floats", () => {
  expect(new NixInt(1n).sub(new NixFloat(2)).toJs()).toBe(-1);
  expect(new NixFloat(2.0).sub(new NixInt(1n)).toJs()).toBe(1);
});

test("'-' operator on non-numbers raises exceptions", () => {
  expect(() => new NixString("foo").sub(new NixFloat(1))).toThrow(
    n.EvalException,
  );
  expect(() => new NixFloat(1).sub(new NixString("foo"))).toThrow(
    n.EvalException,
  );
});

test("'*' operator on integers", () => {
  const result = new NixInt(2n).mul(new NixInt(3n)) as NixInt;
  expect(result.number).toBe(6);
});

test("'*' operator on floats", () => {
  expect(new NixFloat(2.0).mul(new NixFloat(3.5))).toStrictEqual(
    new NixFloat(7),
  );
});

test("'*' operator on mixed integers and floats", () => {
  expect(new NixInt(2n).mul(new NixFloat(3.5))).toStrictEqual(new NixFloat(7));
  expect(new NixFloat(3.5).mul(new NixInt(2n))).toStrictEqual(new NixFloat(7));
});

test("'*' operator on non-numbers raises exceptions", () => {
  expect(() => new NixString("foo").mul(new NixString("bar"))).toThrow(
    n.EvalException,
  );
  expect(() => new NixString("foo").mul(new NixFloat(1))).toThrow(
    n.EvalException,
  );
  expect(() => new NixString("foo").mul(new NixInt(1n))).toThrow(
    n.EvalException,
  );
});

test("'/' operator on integers", () => {
  expect(new NixInt(5n).div(new NixInt(2n))).toStrictEqual(new NixInt(2n));
});

test("'/' operator on floats", () => {
  expect(new NixFloat(5.0).div(new NixFloat(2))).toStrictEqual(
    new NixFloat(2.5),
  );
});

test("'/' operator on mixed integers and floats", () => {
  expect(new NixInt(5n).div(new NixFloat(2.0))).toStrictEqual(
    new NixFloat(2.5),
  );
  expect(new NixFloat(5.0).div(new NixInt(2n))).toStrictEqual(
    new NixFloat(2.5),
  );
});

test("'/' operator on non-numbers raises exceptions", () => {
  expect(() => new NixString("foo").div(new NixString("bar"))).toThrow(
    n.EvalException,
  );
  expect(() => new NixString("foo").div(new NixFloat(1))).toThrow(
    n.EvalException,
  );
  expect(() => new NixString("foo").div(new NixInt(1n))).toThrow(
    n.EvalException,
  );
});

// Attrset:
test("attrset construction", () => {
  expect(attrset(evalCtx(), keyVals()).toJs()).toStrictEqual(new Map());
  expect(
    attrset(evalCtx(), keyVals(["a", new NixFloat(1)])).toJs(),
  ).toStrictEqual(new Map([["a", 1]]));
  const nestedAttrset = new Map([["a", new Map([["b", 1]])]]);
  expect(
    attrset(evalCtx(), keyVals(["a.b", new NixFloat(1)])).toJs(),
  ).toStrictEqual(nestedAttrset);
  expect(
    attrset(
      evalCtx(),
      keyVals(["a", attrset(evalCtx(), keyVals())], ["a.b", new NixFloat(1)]),
    ).toJs(),
  ).toStrictEqual(nestedAttrset);
  expect(
    attrset(
      evalCtx(),
      keyVals(
        ["x.a", attrset(evalCtx(), keyVals())],
        ["x.a.b", new NixFloat(1)],
      ),
    ).toJs(),
  ).toStrictEqual(new Map([["x", nestedAttrset]]));
});

test("attrsets ignore null attrs", () => {
  expect(
    attrset(evalCtx(), (_) => [
      [[n.NULL, new NixString("a")], new NixFloat(1)],
    ]).toJs(),
  ).toStrictEqual(new Map());
  expect(
    attrset(evalCtx(), (_) => [[[n.NULL], new NixFloat(1)]]).toJs(),
  ).toStrictEqual(new Map());
  expect(
    attrset(evalCtx(), (_) => [
      [[new NixString("a"), n.NULL], new NixFloat(1)],
    ]).toJs(),
  ).toStrictEqual(new Map([["a", new Map()]]));
});

test("attrset construction with repeated attrs throws", () => {
  expect(() =>
    attrset(
      evalCtx(),
      keyVals(["a", new NixFloat(1)], ["a", new NixFloat(1)]),
    ).toJs(),
  ).toThrow(new EvalException("Attribute 'a' already defined."));
  expect(() =>
    attrset(
      evalCtx(),
      keyVals(["a", new NixFloat(1)], ["a.b", new NixFloat(2)]),
    ).toJs(),
  ).toThrow(new EvalException("Attribute 'a' already defined."));
  expect(() =>
    attrset(
      evalCtx(),
      keyVals(["a.b", new NixFloat(1)], ["a.b", new NixFloat(2)]),
    ).toJs(),
  ).toThrow(new EvalException("Attribute 'a.b' already defined."));
  expect(() =>
    attrset(
      evalCtx(),
      keyVals(
        ["a.b", attrset(evalCtx(), keyVals(["c", new NixFloat(1)]))],
        ["a.b.c", new NixFloat(2)],
      ),
    ).toJs(),
  ).toThrow(new EvalException("Attribute 'a.b.c' already defined."));
});

test("attrset with non-string attrs throw", () => {
  expect(() =>
    attrset(evalCtx(), (_) => [[[new NixFloat(1)], new NixFloat(1)]]).toJs(),
  ).toThrow(n.EvalException);
});

test("'//' operator on attrsets", () => {
  expect(
    attrset(evalCtx(), keyVals()).update(attrset(evalCtx(), keyVals())).toJs(),
  ).toStrictEqual(new Map());
  expect(
    attrset(evalCtx(), keyVals(["a", new NixFloat(1)]))
      .update(attrset(evalCtx(), keyVals()))
      .toJs(),
  ).toStrictEqual(new Map([["a", 1]]));
  expect(
    attrset(evalCtx(), keyVals(["a", new NixFloat(1)]))
      .update(attrset(evalCtx(), keyVals(["b", new NixFloat(2)])))
      .toJs(),
  ).toStrictEqual(
    new Map([
      ["a", 1],
      ["b", 2],
    ]),
  );
  expect(
    attrset(evalCtx(), keyVals(["a", new NixFloat(1)]))
      .update(attrset(evalCtx(), keyVals(["a", new NixFloat(2)])))
      .toJs(),
  ).toStrictEqual(new Map([["a", 2]]));
});

test("'//' operator on non-attrsets raises exceptions", () => {
  expect(() => attrset(evalCtx(), keyVals()).update(new NixFloat(1))).toThrow(
    n.EvalException,
  );
  expect(() => new NixFloat(1).update(attrset(evalCtx(), keyVals()))).toThrow(
    n.EvalException,
  );
});

test("'?' operator", () => {
  expect(attrset(evalCtx(), keyVals()).has([new NixString("a")])).toBe(n.FALSE);
  expect(
    attrset(evalCtx(), keyVals(["a", new NixFloat(1)])).has([
      new NixString("a"),
    ]),
  ).toBe(n.TRUE);
  expect(
    attrset(evalCtx(), keyVals(["a", new NixFloat(1)])).has([
      new NixString("a"),
      new NixString("b"),
    ]),
  ).toBe(n.FALSE);
  expect(
    attrset(evalCtx(), keyVals(["a.b", new NixFloat(-1)])).has([
      new NixString("a"),
      new NixString("b"),
    ]),
  ).toBe(n.TRUE);
});

test("'?' operator on other types returns false", () => {
  expect(new NixFloat(1).has([new NixString("a")])).toStrictEqual(n.FALSE);
  expect(n.FALSE.has([new NixString("a")])).toStrictEqual(n.FALSE);
});

test("'.' operator", () => {
  expect(
    attrset(evalCtx(), keyVals(["a", new NixFloat(1)]))
      .select([new NixString("a")], undefined)
      .toJs(),
  ).toBe(1);

  expect(
    attrset(evalCtx(), keyVals(["a.b", new NixFloat(1)]))
      .select([new NixString("a"), new NixString("b")], undefined)
      .toJs(),
  ).toBe(1);
  expect(
    attrset(evalCtx(), keyVals()).select([new NixString("a")], new NixFloat(1)),
  ).toStrictEqual(new NixFloat(1));
  expect(
    attrset(evalCtx(), keyVals(["a.a", new NixFloat(1)]))
      .select([new NixString("a"), new NixString("b")], new NixFloat(1))
      .toJs(),
  ).toBe(1);
  expect(
    n
      .attrset(
        evalCtx(),
        keyVals(["a", new NixFloat(1)], ["b.c", new NixFloat(2)]),
      )
      .select([new NixString("a"), new NixString("c")], new NixFloat(5))
      .toJs(),
  ).toBe(5);
});

test("'.' operator throws when attrpath doesn't exist", () => {
  expect(() =>
    attrset(evalCtx(), keyVals()).select([new NixString("a")], undefined),
  ).toThrow(n.EvalException);
});

test("recursive attrsets allow referencing attributes defined later", () => {
  expect(
    n
      .recAttrset(evalCtx(), (ctx) => [
        [
          toAttrpath("a"),
          new Lazy(ctx, (ctx) => ctx.lookup("b").add(new NixFloat(1))),
        ],
        [toAttrpath("b"), new NixFloat(1)],
      ])
      .select([new NixString("a")], undefined)
      .toJs(),
  ).toBe(2);
});

test("recursive attrsets allow referencing attributes from other attribute names", () => {
  expect(
    n
      .recAttrset(evalCtx(), (ctx) => [
        [[new Lazy(ctx, (ctx) => ctx.lookup("a"))], new NixFloat(1)],
        [[new NixString("a")], new NixString("b")],
      ])
      .toJs(),
  ).toStrictEqual(
    new Map<string, any>([
      ["a", "b"],
      ["b", 1],
    ]),
  );
  // This fails in nix but work with our implementation: `rec { ${a} = 1; ${b} = "c"; b = "a"; }`
  expect(
    n
      .recAttrset(evalCtx(), (ctx) => [
        [[new Lazy(ctx, (ctx) => ctx.lookup("a"))], new NixFloat(1)],
        [[new Lazy(ctx, (ctx) => ctx.lookup("b"))], new NixString("c")],
        [[new NixString("b")], new NixString("a")],
      ])
      .toJs(),
  ).toStrictEqual(
    new Map<string, any>([
      ["c", 1],
      ["a", "c"],
      ["b", "a"],
    ]),
  );
});

test("non-recursive attrsets don't allow references to other attributes in the attrset", () => {
  expect(() =>
    n
      .attrset(evalCtx(), (ctx) => [
        [toAttrpath("a"), ctx.lookup("b").add(new NixFloat(1))],
        [toAttrpath("b"), new NixFloat(1)],
      ])
      .select([new NixString("a")], undefined)
      .toJs(),
  ).toThrow(n.EvalException);
});

// Boolean:
test("'&&' operator on booleans", () => {
  expect(n.TRUE.and(n.FALSE)).toBe(n.FALSE);
  expect(n.FALSE.and(new NixFloat(1))).toBe(n.FALSE); // emulates nix's behaviour
});

test("'&&' operator on non-booleans raises exceptions", () => {
  expect(() => n.TRUE.and(new NixFloat(1))).toThrow(n.EvalException);
  expect(() => new NixFloat(1).and(n.TRUE)).toThrow(n.EvalException);
});

test("'->' operator on booleans", () => {
  expect(n.FALSE.implication(n.FALSE)).toBe(n.TRUE);
  expect(n.FALSE.implication(new NixFloat(1))).toBe(n.TRUE); // emulates nix's behaviour
});

test("'->' operator on non-booleans raises exceptions", () => {
  expect(() => n.TRUE.implication(new NixFloat(1))).toThrow(n.EvalException);
  expect(() => new NixFloat(1).implication(n.TRUE)).toThrow(n.EvalException);
});

test("'!' operator on booleans", () => {
  expect(n.FALSE.invert()).toBe(n.TRUE);
});

test("'!' operator on non-booleans raises exceptions", () => {
  expect(() => new NixFloat(1).invert()).toThrow(n.EvalException);
});

test("'||' operator on booleans", () => {
  expect(n.TRUE.or(n.FALSE).toJs()).toBe(true);
  expect(n.TRUE.or(new NixFloat(1)).toJs()).toBe(true); // emulates nix's behaviour
});

test("'||' operator on non-booleans raises exceptions", () => {
  expect(() => n.FALSE.or(new NixFloat(1))).toThrow(n.EvalException);
  expect(() => new NixFloat(1).or(n.TRUE)).toThrow(n.EvalException);
});

// Comparison:
test("'==' operator on numbers", () => {
  expect(new NixFloat(1).eq(new NixFloat(2))).toBe(n.FALSE);
  expect(new NixFloat(1).eq(new NixFloat(1))).toBe(n.TRUE);
  expect(new NixInt(1n).eq(new NixInt(2n))).toBe(n.FALSE);
  expect(new NixInt(1n).eq(new NixInt(1n))).toBe(n.TRUE);
  expect(new NixInt(1n).eq(new NixFloat(1.1))).toBe(n.FALSE);
  expect(new NixInt(1n).eq(new NixFloat(1.0))).toBe(n.TRUE);
  expect(new NixFloat(1.0).eq(new NixInt(1n))).toBe(n.TRUE);
});

test("'==' operator on booleans", () => {
  expect(n.TRUE.eq(n.FALSE)).toBe(n.FALSE);
  expect(n.TRUE.eq(n.TRUE)).toBe(n.TRUE);
});

test("'==' operator on strings", () => {
  expect(new NixString("").eq(new NixString(""))).toBe(n.TRUE);
  expect(new NixString("a").eq(new NixString("b"))).toBe(n.FALSE);
});

test("'==' operator on lists", () => {
  expect(new NixList([]).eq(new NixList([]))).toBe(n.TRUE);
  expect(
    new NixList([new NixFloat(1)]).eq(new NixList([new NixFloat(1)])),
  ).toBe(n.TRUE);
  expect(
    new NixList([new NixList([new NixFloat(1)])]).eq(
      new NixList([new NixList([new NixFloat(1)])]),
    ),
  ).toBe(n.TRUE);
  expect(
    new NixList([new NixFloat(1)]).eq(new NixList([new NixFloat(2)])),
  ).toBe(n.FALSE);
  expect(new NixList([new NixInt(1n)]).eq(new NixList([new NixInt(1n)]))).toBe(
    n.TRUE,
  );
  expect(new NixList([new NixInt(1n)]).eq(new NixList([new NixInt(2n)]))).toBe(
    n.FALSE,
  );
});

test("'==' operator on nulls", () => {
  expect(n.NULL.eq(n.NULL)).toBe(n.TRUE);
  expect(n.NULL.eq(new NixFloat(1))).toBe(n.FALSE);
  expect(new NixString("a").eq(n.NULL)).toBe(n.FALSE);
});

test("'==' operator on attrsets", () => {
  expect(attrset(evalCtx(), keyVals()).eq(attrset(evalCtx(), keyVals()))).toBe(
    n.TRUE,
  );
  expect(
    attrset(evalCtx(), keyVals()).eq(
      attrset(evalCtx(), keyVals(["a", new NixFloat(1)])),
    ),
  ).toBe(n.FALSE);
  expect(
    attrset(evalCtx(), keyVals(["a", new NixFloat(1)])).eq(
      attrset(evalCtx(), keyVals(["a", new NixFloat(1)])),
    ),
  ).toBe(n.TRUE);
  expect(
    attrset(evalCtx(), keyVals(["a", new NixFloat(1)])).eq(
      attrset(evalCtx(), keyVals(["a", new NixFloat(2)])),
    ),
  ).toBe(n.FALSE);
});

test("'!=' operator on floats", () => {
  expect(new NixFloat(1).neq(new NixFloat(2))).toBe(n.TRUE);
  expect(new NixFloat(1).neq(new NixFloat(1))).toBe(n.FALSE);
});

test("'<' operator on numbers", () => {
  expect(new NixFloat(1).less(new NixFloat(2))).toBe(n.TRUE);
  expect(new NixInt(1n).less(new NixInt(2n))).toBe(n.TRUE);
  expect(new NixInt(1n).less(new NixFloat(2))).toBe(n.TRUE);
  expect(new NixFloat(1).less(new NixInt(2n))).toBe(n.TRUE);
});

test("'<' operator on mixed-types throws", () => {
  expect(() => new NixInt(1n).less(n.TRUE)).toThrow(n.EvalException);
  expect(() => n.TRUE.less(new NixInt(1n))).toThrow(n.EvalException);
  expect(() => n.TRUE.less(new NixFloat(1))).toThrow(n.EvalException);
});

test("'<' operator on strings", () => {
  expect(new NixString("a").less(new NixString("b"))).toBe(n.TRUE);
  expect(new NixString("foo").less(new NixString("b"))).toBe(n.FALSE);
});

test("'<' operator on booleans throws", () => {
  expect(() => n.FALSE.less(n.TRUE)).toThrow(n.EvalException);
});

test("'<' operator on null values throws", () => {
  expect(() => n.NULL.less(n.NULL)).toThrow(n.EvalException);
});

test("'<' operator on lists", () => {
  expect(new NixList([]).less(new NixList([]))).toBe(n.FALSE);
  expect(new NixList([]).less(new NixList([new NixFloat(1)]))).toBe(n.TRUE);
  expect(new NixList([new NixFloat(1)]).less(new NixList([]))).toBe(n.FALSE);
  expect(
    new NixList([new NixFloat(1)]).less(
      new NixList([new NixFloat(1), new NixFloat(2)]),
    ),
  ).toBe(n.TRUE);
  expect(
    new NixList([new NixFloat(1), new NixFloat(2)]).less(
      new NixList([new NixFloat(1)]),
    ),
  ).toBe(n.FALSE);
  expect(
    new NixList([new NixFloat(1), new NixFloat(1)]).less(
      new NixList([new NixFloat(1), new NixFloat(2)]),
    ),
  ).toBe(n.TRUE);
  expect(
    new NixList([new NixFloat(1), n.TRUE]).less(new NixList([new NixFloat(1)])),
  ).toBe(n.FALSE);
  expect(
    new NixList([new NixInt(1n)]).less(new NixList([new NixInt(2n)])),
  ).toBe(n.TRUE);

  // This reproduces nix's observed behaviour
  expect(new NixList([n.TRUE]).less(new NixList([n.TRUE]))).toBe(n.FALSE);
  expect(new NixList([n.FALSE]).less(new NixList([n.FALSE]))).toBe(n.FALSE);
  expect(
    new NixList([n.FALSE, new NixFloat(1)]).less(
      new NixList([n.FALSE, new NixFloat(2)]),
    ),
  ).toBe(n.TRUE);
  expect(new NixList([n.NULL]).less(new NixList([n.NULL]))).toBe(n.FALSE);
});

test("'<' operator on lists with lazy values", () => {
  expect(
    new NixList([new Lazy(evalCtx(), (_) => new NixFloat(1))]).less(
      new NixList([new Lazy(evalCtx(), (_) => new NixFloat(1))]),
    ),
  ).toBe(n.FALSE);

  expect(
    new NixList([new Lazy(evalCtx(), (_) => new NixFloat(1))]).less(
      new NixList([new Lazy(evalCtx(), (_) => new NixFloat(2))]),
    ),
  ).toBe(n.TRUE);

  expect(
    new NixList([new Lazy(evalCtx(), (_) => n.TRUE)]).less(
      new NixList([new Lazy(evalCtx(), (_) => n.TRUE)]),
    ),
  ).toBe(n.FALSE);
});

test("'<' operator list invalid", () => {
  expect(() =>
    new NixList([n.TRUE]).less(new NixList([new NixFloat(1)])),
  ).toThrow(n.EvalException);
  expect(() => new NixList([n.TRUE]).less(new NixList([n.FALSE]))).toThrow(
    n.EvalException,
  );
});

test("'<' operator on attrsets invalid", () => {
  let smallAttrset = n.attrset(evalCtx(), keyVals(["a", new NixFloat(1)]));
  expect(() => smallAttrset.less(smallAttrset)).toThrow(n.EvalException);
});

test("'<' operator on paths", () => {
  expect(new Path("./a").less(new Path("./b"))).toStrictEqual(n.TRUE);
  expect(new Path("./a").less(new Path("./a"))).toStrictEqual(n.FALSE);
});

test("'<=' operator", () => {
  expect(new NixFloat(1).lessEq(new NixFloat(0))).toBe(n.FALSE);
  expect(new NixFloat(1).lessEq(new NixFloat(1))).toBe(n.TRUE);
  expect(new NixFloat(1).lessEq(new NixFloat(2))).toBe(n.TRUE);

  // This reproduces nix's observed behaviour
  expect(new NixList([n.TRUE]).lessEq(new NixList([n.TRUE]))).toBe(n.TRUE);
  expect(new NixList([n.NULL]).lessEq(new NixList([n.NULL]))).toBe(n.TRUE);
});

test("'>=' operator", () => {
  expect(new NixFloat(1).moreEq(new NixFloat(0))).toBe(n.TRUE);
  expect(new NixFloat(1).moreEq(new NixFloat(1))).toBe(n.TRUE);
  expect(new NixFloat(1).moreEq(new NixFloat(2))).toBe(n.FALSE);

  // This reproduces nix's observed behaviour
  expect(new NixList([n.TRUE]).moreEq(new NixList([n.TRUE]))).toBe(n.TRUE);
  expect(new NixList([n.NULL]).moreEq(new NixList([n.NULL]))).toBe(n.TRUE);
});

test("'>' operator", () => {
  expect(new NixFloat(1).more(new NixFloat(0))).toBe(n.TRUE);
  expect(new NixFloat(1).more(new NixFloat(1))).toBe(n.FALSE);
  expect(new NixFloat(1).more(new NixFloat(2))).toBe(n.FALSE);
});

// Lambda:
test("parameter lambda", () => {
  expect(
    n
      .paramLambda(evalCtx(), "foo", (evalCtx) => evalCtx.lookup("foo"))
      .apply(n.TRUE),
  ).toBe(n.TRUE);
});

test("pattern lambda", () => {
  const arg = n.attrset(evalCtx(), keyVals(["a", new NixFloat(1)]));
  expect(
    n
      .patternLambda(evalCtx(), undefined, [["a", undefined]], (evalCtx) =>
        evalCtx.lookup("a"),
      )
      .apply(arg)
      .toJs(),
  ).toBe(1);
});

test("pattern lambda with default values", () => {
  const arg = n.attrset(evalCtx(), keyVals());
  expect(
    n
      .patternLambda(evalCtx(), undefined, [["a", 1]], (evalCtx) =>
        evalCtx.lookup("a"),
      )
      .apply(arg),
  ).toBe(1);
});

test("pattern lambda with missing parameter", () => {
  let innerCtx = evalCtx().withShadowingScope(
    n.attrset(evalCtx(), keyVals(["a", new NixFloat(1)])),
  );
  expect(() =>
    n
      .patternLambda(innerCtx, undefined, [["a", undefined]], (evalCtx) =>
        evalCtx.lookup("a"),
      )
      .apply(n.attrset(evalCtx(), keyVals())),
  ).toThrow(n.EvalException);
});

test("pattern lambda with arguments binding", () => {
  const arg = n.attrset(evalCtx(), keyVals(["a", new NixFloat(1)]));
  expect(
    n
      .patternLambda(evalCtx(), "args", [["a", undefined]], (evalCtx) =>
        evalCtx.lookup("args").select([new NixString("a")], undefined),
      )
      .apply(arg)
      .toJs(),
  ).toBe(1);
});

// Lazy:
test("'Lazy.toStrict' evaluates the body only once", () => {
  let sentinel = new NixFloat(0);
  let lazyValue = new Lazy(evalCtx(), (_) => {
    sentinel = sentinel.add(new NixFloat(1)) as NixFloat;
    return sentinel;
  });
  expect(lazyValue.toStrict().toJs()).toEqual(1);
  expect(lazyValue.toStrict().toJs()).toEqual(1);
});

test("'Lazy.toStrict' uses the construction-time evaluation context", () => {
  const innerValue = new NixFloat(42);
  let innerCtx = evalCtx().withShadowingScope(
    n.attrset(evalCtx(), keyVals(["a", innerValue])),
  );
  let lazyValue = new Lazy(innerCtx, (evalCtx) => evalCtx.lookup("a"));
  expect(lazyValue.toStrict()).toEqual(innerValue);
});

test("'Lazy.toStrict' drops the body and the evaluation context", () => {
  let lazyValue = new Lazy(evalCtx(), (_) => n.TRUE);
  lazyValue.toStrict();
  expect(lazyValue.body).toBeUndefined();
  expect(lazyValue.evalCtx).toBeUndefined();
});

// List:
test("'++' operator", () => {
  const list_1 = new NixList([new NixFloat(1)]);
  const list_2 = new NixList([new NixFloat(2)]);
  expect(list_1.concat(list_2)).toStrictEqual(
    new NixList([new NixFloat(1), new NixFloat(2)]),
  );
  // Here's we're verifying that neither of the operands is mutated.
  expect(list_1).toStrictEqual(new NixList([new NixFloat(1)]));
  expect(list_2).toStrictEqual(new NixList([new NixFloat(2)]));
});

test("'++' operator on lazy lists with lazy values", () => {
  expect(
    new NixList([new Lazy(evalCtx(), (_) => new NixFloat(1))])
      .concat(
        new Lazy(
          evalCtx(),
          (_) => new NixList([new Lazy(evalCtx(), (_) => new NixFloat(2))]),
        ),
      )
      .toJs(),
  ).toStrictEqual([1, 2]);
});

test("'++' operator on non-lists raises exceptions", () => {
  expect(() => new NixList([]).concat(new NixFloat(1))).toThrow(
    n.EvalException,
  );
  expect(() => n.TRUE.concat(new NixList([]))).toThrow(n.EvalException);
});

// Path:
test("toPath on absolute paths", () => {
  expect(n.toPath(evalCtx(), "/a")).toStrictEqual(new Path("/a"));
  expect(n.toPath(evalCtx(), "/./a/../b")).toStrictEqual(new Path("/b"));
  expect(n.toPath(evalCtx(), "//./a//..///b/")).toStrictEqual(new Path("/b"));
});

test("toPath transforms relative paths with 'joinPaths'", () => {
  expect(n.toPath(evalCtx(), "a")).toStrictEqual(new Path("/test_base/a"));
  expect(n.toPath(evalCtx(), "./a")).toStrictEqual(new Path("/test_base/a"));
});

// Scope:
test("variable not in global scope", () => {
  expect(() => evalCtx().lookup("foo")).toThrow(n.EvalException);
});

test("variable in shadowing scope", () => {
  expect(
    evalCtx()
      .withShadowingScope(n.attrset(evalCtx(), keyVals(["foo", n.TRUE])))
      .lookup("foo")
      .toJs(),
  ).toBe(true);
});

// Type functions:
test("typeOf", () => {
  expect(new NixInt(1n).typeOf()).toBe("int");
  expect(new NixFloat(5.0).typeOf()).toBe("float");
  expect(new NixString("a").typeOf()).toBe("string");
  expect(n.TRUE.typeOf()).toBe("bool");
  expect(n.NULL.typeOf()).toBe("null");
  expect(new NixList([]).typeOf()).toBe("list");
  expect(attrset(evalCtx(), keyVals()).typeOf()).toBe("set");
  expect(new Path("/").typeOf()).toBe("path");
  expect(new Lambda((_) => n.TRUE).typeOf()).toBe("lambda");
  // TODO: cover other Nix types
});

// With:
test("'with' expression puts attrs into scope", () => {
  const namespace = n.attrset(evalCtx(), keyVals(["a", new NixFloat(1)]));
  expect(
    n.withExpr(evalCtx(), namespace, (evalCtx) => evalCtx.lookup("a")).toJs(),
  ).toBe(1);
});

test("'with' expression does not shadow variables", () => {
  const namespace = n.attrset(evalCtx(), keyVals(["a", new NixFloat(1)]));
  let outerCtx = evalCtx().withShadowingScope(
    n.attrset(evalCtx(), keyVals(["a", new NixFloat(2)])),
  );
  expect(n.withExpr(outerCtx, namespace, (ctx) => ctx.lookup("a")).toJs()).toBe(
    2,
  );
});

test("'with' expressions shadow themselves", () => {
  const outerNamespace = n.attrset(evalCtx(), keyVals(["a", new NixFloat(1)]));
  const innerNamespace = n.attrset(evalCtx(), keyVals(["a", new NixFloat(2)]));
  const innerExpr = (ctx) =>
    n.withExpr(ctx, innerNamespace, (ctx) => ctx.lookup("a"));
  expect(n.withExpr(evalCtx(), outerNamespace, innerExpr).toJs()).toBe(2);
});
