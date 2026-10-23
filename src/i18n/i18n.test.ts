import { expect, test } from "vitest";
import en from "./en.json";
import fr from "./fr.json";

function keys(value: unknown, prefix = ""): string[] {
  if (typeof value !== "object" || value === null) return [prefix];
  return Object.entries(value).flatMap(([k, v]) => keys(v, prefix ? `${prefix}.${k}` : k));
}

test("le français et l'anglais ont les mêmes clés", () => {
  expect(keys(en).sort()).toEqual(keys(fr).sort());
});
