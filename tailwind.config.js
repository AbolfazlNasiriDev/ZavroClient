/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        adwaita: {
          bg: "#2e2e2e",
          panel: "#3a3a3a",
          panel2: "#444444",
          border: "#4a4a4a",
          text: "#eeeeec",
          muted: "#9e9e9e",
          accent: "#3584e4",
          accentHover: "#2a76d2",
          green: "#33d17a",
          yellow: "#f6d32d",
          red: "#e01b24",
        },
      },
    },
  },
  plugins: [],
};
