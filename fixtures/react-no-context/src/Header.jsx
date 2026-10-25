import React from "react";
import { ThemeContext } from "./theme";

export function Header() {
  const theme = React.useContext(ThemeContext);
  return <h1 className={theme}>Title</h1>;
}
