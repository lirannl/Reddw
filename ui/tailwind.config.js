/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {},
  },
  plugins: [require("daisyui")],
  safelist: [
    "alert-error",
    "alert-neutral",
    "alert-info",
    "alert-warning"
  ]
}

