export const range = (min, max) =>
  Array.from({ length: max - min }, (x, i) => min + i);
