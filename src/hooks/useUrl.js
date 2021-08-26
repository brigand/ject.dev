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

  /**
   * With no arguments, returns the URLSearchParams instance.
   * Pass a key to get a specific query param value string.
   */
  query(key = null) {
    return key == null ? this.#params : this.#params.get(key);
  }

  /**
   * With no arguments, returns the state object.
   * Pass a key to get a specific state field value.
   */
  state(key = null) {
    return key == null ? this.#location.state : this.#location.state?.[key];
  }

  /**
   * Replaces the pathname portion of the location object.
   * Returns an updated copy.
   */
  withPath(pathname) {
    const location = { ...this.#location, pathname };
    return new UrlWrapper(location, this.#history, this.#params);
  }

  /**
   * Set or delete (pass null for value) a specific query param.
   * Returns an updated copy.
   */
  withQuery(k, v) {
    const params = new URLSearchParams(this.#params);
    if (v == null) {
      params.delete(k);
    } else {
      params.set(k, v);
    }

    return new UrlWrapper(this.#location, this.#history, params);
  }

  /**
   * Resets state to an empty object.
   * Returns an updated copy.
   */
  withEmptyState() {
    const location = {
      ...this.#location,
      state: {},
    };
    return new UrlWrapper(location, this.#history, this.#params);
  }

  /**
   * Set a specific state field.
   * Returns an updated copy.
   */
  withState(k, v) {
    const location = {
      ...this.#location,
      state: { ...this.#location.state, [k]: v },
    };
    return new UrlWrapper(location, this.#history, this.#params);
  }

  /**
   * Builds a react-router location object.
   */
  buildLocation() {
    const search = this.#params.toString();
    return { ...this.#location, search: search ? '?' + search : '' };
  }

  /**
   * Pushes the location onto the react-router history. Typical end of a chain.
   */
  applyByPush() {
    this.#history.push(this.buildLocation());
    return this;
  }

  /**
   * Replace the top location in the react-router history. Typical end of a chain.
   */
  applyByReplace() {
    this.#history.replace(this.buildLocation());
    return this;
  }
}

/**
 * Converts the react-router location to a fluent interface for inspecting/modifying the path/query/state.
 *
 * @returns UrlWrapper
 */
function useUrl() {
  const location = useLocation();
  const history = useHistory();

  return new UrlWrapper(location, history);
}

export default useUrl;
