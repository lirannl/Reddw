/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    minWidth: {
      '3/4': '75%',
      '1/2': '50%',
      '1/4': '25%',
    },
    extend: {},
  },
  plugins: [require("daisyui")],
}

