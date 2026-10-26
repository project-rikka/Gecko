/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

const lazy = {};

ChromeUtils.defineESModuleGetters(lazy, {
  DownloadPaths: "resource://gre/modules/DownloadPaths.sys.mjs",
  Downloads: "resource://gre/modules/Downloads.sys.mjs",
  EventDispatcher: "resource://gre/modules/Messaging.sys.mjs",
});

const FALLBACK_NAME = "download";

function fileNameOf(uri) {
  let name = "";
  try {
    name = decodeURIComponent(uri.QueryInterface(Ci.nsIURL).fileName);
  } catch {}
  return lazy.DownloadPaths.sanitize(name) || FALLBACK_NAME;
}

function sourceFor(browser, uri) {
  const context = browser.browsingContext;
  const global = context?.currentWindowGlobal;
  if (!global) {
    throw new Error("The page has no document yet");
  }
  const referrerInfo = Cc["@mozilla.org/referrer-info;1"].createInstance(Ci.nsIReferrerInfo);
  referrerInfo.init(Ci.nsIReferrerInfo.EMPTY, true, browser.currentURI);
  return {
    url: uri.spec,
    isPrivate: context.usePrivateBrowsing,
    userContextId: context.originAttributes.userContextId,
    browsingContextId: context.id,
    loadingPrincipal: global.documentPrincipal,
    cookieJarSettings: global.cookieJarSettings,
    referrerInfo,
  };
}

export async function saveUri(window, browser, spec) {
  if (typeof spec !== "string" || !spec) {
    throw new Error("A download needs a URI");
  }
  const uri = Services.io.newURI(spec);
  const source = sourceFor(browser, uri);
  const answer = await lazy.EventDispatcher.for(window).sendRequestForResult("Rikka:DownloadRequest", {
    url: uri.spec,
    filename: fileNameOf(uri),
    mimeType: null,
    contentLength: null,
    reason: "save",
  });
  const path = answer?.path;
  if (typeof path !== "string" || !PathUtils.isAbsolute(path)) {
    return;
  }
  const download = await lazy.Downloads.createDownload({ source, target: path });
  const list = await lazy.Downloads.getList(source.isPrivate ? lazy.Downloads.PRIVATE : lazy.Downloads.PUBLIC);
  await list.add(download);
  download.start().catch(error => console.error(error));
}
