module.exports = {
 purge: {
  content: [
   "./src/**/*.html",
   "./src/**/*.js",
   "./src/**/*.jsx",
   "./src/**/*.ts",
   "./src/**/*.tsx",
   "./public/**/*.html",
  ],
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
  defaultLineHeights: true,
 },
 plugins: [require("@tailwindcss/ui")],
};
