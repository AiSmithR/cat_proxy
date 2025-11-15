/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#f5f7ff',
          100: '#ebefff',
          200: '#d6deff',
          300: '#b8c4ff',
          400: '#9aa5ff',
          500: '#667eea',
          600: '#5568d3',
          700: '#4553b8',
          800: '#364098',
          900: '#2a3378',
        },
        secondary: {
          500: '#764ba2',
          600: '#653d8a',
          700: '#552f72',
        }
      },
      fontFamily: {
        sans: ['-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'sans-serif'],
        mono: ['Monaco', 'Courier New', 'monospace'],
      },
    },
  },
  plugins: [],
}
