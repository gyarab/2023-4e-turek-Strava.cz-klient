/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  darkMode: 'class',
  theme: {
    //extend: { opacity: ['disabled'],},
    borderWidth: {
      DEFAULT: '1px',
      0: '0',
      2: '1.5px',
      3: '3px',
      4: '4px',
      8: '8px',
    },
  },
  plugins: [require('tailwind-scrollbar')({ nocompatible: true })],

}

