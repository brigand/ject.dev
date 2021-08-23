import { default$ } from './default';
import { react } from './react';

export const byName = new Map([
  ['default', default$],
  ['react', react],
]);

export function clone({ files, ...rest }) {
  return {
    ...rest,
    files: files.map((file) => ({ version: 1, ...file })),
  };
}

export function get(name = 'default') {
  const template = byName.get(name);
  return template ? clone(template) : null;
}
