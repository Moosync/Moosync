async function getBlobUrl(src) {
  const resp = await fetch(src);
  const blob = await resp.blob();
  const url = URL.createObjectURL(blob);
  return url;
}

window.__MOOSYNC__ = {
  ...window.__MOOSYNC__,
  getBlobUrl,
};
