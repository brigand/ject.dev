import { useLocation, useHistory } from 'react-router-dom';

export class UrlWrapper {
  #location;
  #history;
  #params;
  constructor(location, history, searchParams = null) {
    this.#location = location;
    this.#history = history;
    this.#params =
      searchParams ?? new URLSearchParams(location.search?.replace('?', '') ?? '');
  }

  query(key = null) {
    return key == null ? this.#params : this.#params.get(key);
  }

  withPath(pathname) {
    const location = { ...this.#location, pathname };
    new UrlWrapper(location, this.#history);
  }

  withQuery(k, v) {
    const params = new URLSearchParams(this.#params);
    params.set(k, v);
    return new UrlWrapper(this.#location, this.#history, params);
  }

  withEmptyState() {
    const location = {
      ...this.#location,
      state: {},
    };
    return new UrlWrapper(location, this.#history, this.#params);
  }

  withState(k, v) {
    const location = {
      ...this.#location,
      state: { ...this.#location.state, [k]: v },
    };
    return new UrlWrapper(location, this.#history, this.#params);
  }

  buildLocation() {
    const search = this.#params.toString();
    return { ...this.#location, search: '?' + search };
  }

  applyByPush() {
    this.history.push(this.buildLocation());
    return this;
  }

  applyByReplace() {
    this.history.replace(this.buildLocation());
    return this;
  }
}

function useUrl() {
  const location = useLocation();
  const history = useHistory();

  return new UrlWrapper(location, history);
}

export default useUrl;
