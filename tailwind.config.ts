import type { Config } from "tailwindcss";

export default {
  darkMode: ["class"],
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        canvas: "hsl(var(--canvas))",
        panel: "hsl(var(--panel))",
        line: "hsl(var(--line))",
        ink: "hsl(var(--ink))",
        muted: "hsl(var(--muted))",
        sage: {
          50: "#effdf7",
          100: "#d9f8ea",
          300: "#72d7ad",
          400: "#3abb8b",
          500: "#219d72",
          600: "#167e5c",
          900: "#103f32"
        }
      },
      fontFamily: {
        sans: ["Inter", "ui-sans-serif", "system-ui", "sans-serif"],
        mono: ["SFMono-Regular", "Cascadia Code", "ui-monospace", "monospace"]
      },
      boxShadow: {
        soft: "0 24px 80px rgba(0, 0, 0, 0.22)"
      }
    }
  },
  plugins: []
} satisfies Config;

