module.exports = {
 purge: {
  // mode: "all",
  content: [
   "./src/**/*.html",
   "./src/**/*.js",
   "./src/**/*.jsx",
   "./src/**/*.ts",
   "./src/**/*.tsx",
   "./public/**/*.html",
  ],
  // options: {
  //  whitelist: ["textarea"],
  // },
 },
 theme: {
  extend: {
   transitionProperty: {
    "color-transform": "color, transform",
   },
  },
 },
 variants: {
  translate: ["responsive", "hover", "focus", "group-hover"],
 },
 future: {
  removeDeprecatedGapUtilities: true,
  purgeLayersByDefault: true,
  standardFontWeights: true,
  // defaultLineHeights: true,
 },
 // experimental: {
 //  uniformColorPalette: true,
 // },
 plugins: [require("@tailwindcss/ui")],
};
