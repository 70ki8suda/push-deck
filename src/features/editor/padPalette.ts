import type { PadColorId } from "../../lib/types";

export const padColors: Record<
  PadColorId,
  { background: string; foreground: string }
> = {
  off: {
    background: "linear-gradient(145deg, #303030 0%, #121212 100%)",
    foreground: "#c6c6c6",
  },
  white: {
    background: "linear-gradient(145deg, #fffaf2 0%, #ddd4c8 100%)",
    foreground: "#171717",
  },
  peach: {
    background: "linear-gradient(145deg, #f6c0d5 0%, #e58cb0 100%)",
    foreground: "#2c1020",
  },
  coral: {
    background: "linear-gradient(145deg, #ffb0a2 0%, #ef6f61 100%)",
    foreground: "#2f110e",
  },
  red: {
    background: "linear-gradient(145deg, #ff7b74 0%, #d72638 100%)",
    foreground: "#fff5f5",
  },
  orange: {
    background: "linear-gradient(145deg, #ffbd72 0%, #e9782f 100%)",
    foreground: "#fff8f1",
  },
  amber: {
    background: "linear-gradient(145deg, #ffe7a3 0%, #f3bf3a 100%)",
    foreground: "#2b2000",
  },
  yellow: {
    background: "linear-gradient(145deg, #fff486 0%, #f4dc3f 100%)",
    foreground: "#272300",
  },
  lime: {
    background: "linear-gradient(145deg, #d7f59c 0%, #9cd94b 100%)",
    foreground: "#162900",
  },
  chartreuse: {
    background: "linear-gradient(145deg, #b8ef6e 0%, #6cbc2f 100%)",
    foreground: "#102400",
  },
  green: {
    background: "linear-gradient(145deg, #67d98a 0%, #1f9d55 100%)",
    foreground: "#f4fff7",
  },
  mint: {
    background: "linear-gradient(145deg, #a2f0d0 0%, #54c79b 100%)",
    foreground: "#032219",
  },
  teal: {
    background: "linear-gradient(145deg, #8ee3d1 0%, #24b39a 100%)",
    foreground: "#04211a",
  },
  cyan: {
    background: "linear-gradient(145deg, #90def2 0%, #33b5d9 100%)",
    foreground: "#041b24",
  },
  sky: {
    background: "linear-gradient(145deg, #93c5fd 0%, #3b82f6 100%)",
    foreground: "#f2f7ff",
  },
  blue: {
    background: "linear-gradient(145deg, #6d8dff 0%, #2563eb 100%)",
    foreground: "#f3f7ff",
  },
  indigo: {
    background: "linear-gradient(145deg, #9d8cff 0%, #5b5bd6 100%)",
    foreground: "#f6f3ff",
  },
  purple: {
    background: "linear-gradient(145deg, #b18cff 0%, #7c3aed 100%)",
    foreground: "#f6f1ff",
  },
  magenta: {
    background: "linear-gradient(145deg, #f29cff 0%, #d946ef 100%)",
    foreground: "#fff2ff",
  },
  rose: {
    background: "linear-gradient(145deg, #f8a8c7 0%, #ec4899 100%)",
    foreground: "#fff2f7",
  },
  pink: {
    background: "linear-gradient(145deg, #ff8fbd 0%, #db2777 100%)",
    foreground: "#fff2f7",
  },
};
