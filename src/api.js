const parseBody = async (res) => {
  const ct = res.headers.get('content-type');
  if (ct && ct.toLowerCase().startsWith('application/json')) {
    return res.json();
  } else {
    return res.text();
  }
};
const fetch2 = async (url, { json, ...options }) => {
  if (json != null) {
    options.body = JSON.stringify(json);
    options.headers = { ...options.headers, 'content-type': 'application/json' };
  }
  const res = await fetch(url, options);
  const body = parseBody(res);

  if (res.ok) {
    return body;
  } else {
    let message =
      typeof body === 'string'
        ? body
        : body && typeof body.message === 'string'
        ? body.message
        : 'Unknown';
    throw Object.assign(
      new Error(
        `Request ${(options.method || 'get').toUpperCase()} ${url} error ${
          res.status
        }: ${message}`,
      ),
      { status: res.status, body },
    );
  }
};

export async function createSession(session) {
  return fetch2(`/api/session/new`, { method: 'POST', json: { session } });
}

export async function updateSession(session_id, session) {
  return fetch2(`/api/session`, { method: 'PUT', json: { session_id, session } });
}

export async function save(session) {
  return fetch2(`/api/save`, { method: 'POST', json: { session } });
}

export async function getSaved(save_id) {
  return fetch2(`/api/saved/${encodeURIComponent(save_id)}`, { method: 'GET' });
}
