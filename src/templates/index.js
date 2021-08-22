import { default$ } from './default';

export const byName = new Map([['default', default$]]);

export function clone({ files, ...rest }) {
  return {
    ...rest,
    files: files.map((file) => ({ version: 1, ...file })),
  };
}

export function get(name = 'default') {
  return clone(byName.get(name));
}
