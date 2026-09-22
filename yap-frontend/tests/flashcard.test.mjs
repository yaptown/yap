import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import vm from "node:vm";
import ts from "typescript";

// Exercise the actual menu handlers and reveal state, not a copy of the gate.
// Like pending-review.test.mjs, rendering/browser dependencies are stubbed.
const source = ts.transpileModule(
  readFileSync(new URL("../src/components/Flashcard.tsx", import.meta.url), "utf8"),
  { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022, jsx: ts.JsxEmit.ReactJSX } },
).outputText;

function harness(requireReveal) {
  const slots = [];
  const effects = [];
  const ratings = [];
  let cursor = 0;
  const noop = () => {};
  const jsx = (type, props) => ({ type, props });
  const exports = {};
  vm.runInNewContext(source, {
    exports,
    window: { addEventListener: noop, removeEventListener: noop, scrollTo: noop },
    require(path) {
      if (path.includes("yap-frontend-rs/pkg")) throw Error("Flashcard must not import WASM values");
      if (path === "react/jsx-runtime") return { jsx, jsxs: jsx };
      if (path === "react") return {
        useState(initial) {
          const index = cursor++;
          if (!(index in slots)) slots[index] = initial;
          return [slots[index], (value) => { slots[index] = typeof value === "function" ? value(slots[index]) : value; }];
        },
        useCallback: (fn) => fn,
        useEffect: (fn) => effects.push(fn),
      };
      if (path === "framer-motion") return {
        motion: { div: "motion.div" },
        useMotionValue: noop,
        useTransform: noop,
        useAnimation: () => ({ set: noop, start: noop }),
      };
      if (path === "./background-context") return { useBackground: () => ({ bumpBackground: noop }) };
      if (path === "@/lib/utils") return { cn: (...values) => values.join(" ") };
      // JSX is retained as a tree; child components are not executed.
      return new Proxy({}, { get: (_, name) => name });
    },
  });
  const props = {
    content: { type: "Gram", gram: [], definition: { is_phrase: false, senses: [] } },
    view: {
      require_answer_reveal: requireReveal,
      again_label: "Forgot", remembered_label: "Remembered", reveal_label: "Show English",
      menu_grades: ["easy", "good", "hard"].map((rating) => ({ rating, label: rating.toUpperCase() })),
    },
    onRating: (rating) => ratings.push(rating),
  };
  const render = () => {
    cursor = 0;
    effects.length = 0;
    const tree = exports.Flashcard(props);
    effects.forEach((effect) => effect());
    return tree;
  };
  return { ratings, render };
}

function nodes(tree, predicate) {
  if (!tree || typeof tree !== "object") return [];
  if (Array.isArray(tree)) return tree.flatMap((child) => nodes(child, predicate));
  return [...(predicate(tree) ? [tree] : []), ...nodes(tree.props?.children, predicate)];
}
const menuGrades = (tree) => nodes(tree, (node) => node.type === "DropdownMenuItem" && "disabled" in node.props);
const card = (tree) => nodes(tree, (node) => node.type === "Card")[0];

for (const [index, rating] of ["easy", "good", "hard"].entries()) {
  test(`${rating} cannot grade before reveal, then can grade after reveal and rehide`, () => {
    const h = harness(true);
    let tree = h.render();
    const grades = menuGrades(tree);
    assert.deepEqual(grades.map((item) => item.props.children), ["EASY", "GOOD", "HARD"]);
    for (const grade of grades) {
      assert.equal(grade.props.disabled, true);
      grade.props.onClick(); // Even a direct/programmatic invocation is guarded.
    }
    assert.deepEqual(h.ratings, []);
    card(tree).props.onClick();
    tree = h.render();
    assert.ok(menuGrades(tree).every((item) => item.props.disabled === false));
    card(tree).props.onClick();
    tree = h.render();
    assert.equal(menuGrades(tree)[index].props.disabled, false, "a previously opened card remains gradeable");
    menuGrades(tree)[index].props.onClick();
    assert.deepEqual(h.ratings, [rating]);
  });

  test(`${rating} can grade immediately when reveal is not required`, () => {
    const h = harness(false);
    const grade = menuGrades(h.render())[index];
    assert.equal(grade.props.disabled, false);
    grade.props.onClick();
    assert.deepEqual(h.ratings, [rating]);
  });
}
