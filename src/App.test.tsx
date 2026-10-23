import { render, screen } from "@testing-library/react";
import { expect, test } from "vitest";
import App from "./App";

test("affiche l'application", () => {
  render(<App />);
  expect(screen.getByRole("main")).toBeInTheDocument();
});
