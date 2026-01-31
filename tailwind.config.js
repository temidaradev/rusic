/** @type {import('tailwindcss').Config} */
module.exports = {
  mode: "all",
  content: [
    "./rusic/**/*.{rs,html,css}",
    "./components/**/*.{rs,html,css}",
    "./pages/**/*.{rs,html,css}",
    "./hooks/**/*.{rs,html,css}",
    "./player/**/*.{rs,html,css}",
    "./reader/**/*.{rs,html,css}",
    "./server/**/*.{rs,html,css}",
    "./utils/**/*.{rs,html,css}",
    "./config/**/*.{rs,html,css}",
    "./rusic_route/**/*.{rs,html,css}",
  ],
  theme: {
    extend: {},
  },
  plugins: [],
};
