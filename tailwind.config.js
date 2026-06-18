/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        brand: {
          DEFAULT: '#5C7AFF', // Primary Brand Color
          dark: '#4B63D1',
          light: '#7B94FF',
        },
        surface: {
          DEFAULT: '#FFFFFF',
          secondary: '#F3F4F6', // Neutral background surface (60%)
          elevated: '#E5E7EB', // Cards, panels (30%)
        },
        dark: {
          surface: '#121212',
          secondary: '#1E1E1E',
          elevated: '#2A2A2A',
        }
      },
      spacing: {
        '1': '4px',
        '2': '8px',
        '3': '12px',
        '4': '16px',
        '6': '24px',
        '8': '32px',
        '12': '48px',
      },
      borderRadius: {
        'sm': '4px',
        'md': '8px',
        'lg': '12px',
      },
      fontFamily: {
        sans: ['Inter', 'Segoe UI', 'SF Pro', 'system-ui', 'sans-serif'],
      },
      transitionDuration: {
        'fast': '150ms',
        'standard': '250ms',
        'complex': '300ms',
      }
    },
  },
  plugins: [],
}
