/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

const lazy = {};

ChromeUtils.defineESModuleGetters(lazy, {
  NetUtil: "resource://gre/modules/NetUtil.sys.mjs",
});

function channelFor(browser, uri) {
  const global = browser.browsingContext?.currentWindowGlobal;
  if (!global) {
    throw new Error("The page has no document yet");
  }
  const channel = lazy.NetUtil.newChannel({
    uri: Services.io.newURI(uri),
    loadingPrincipal: global.documentPrincipal,
    contentPolicyType: Ci.nsIContentPolicy.TYPE_SAVEAS_DOWNLOAD,
    securityFlags: Ci.nsILoadInfo.SEC_ALLOW_CROSS_ORIGIN_INHERITS_SEC_CONTEXT,
  });
  if (channel instanceof Ci.nsIPrivateBrowsingChannel) {
    channel.setPrivate(browser.browsingContext.usePrivateBrowsing);
  }
  channel.loadFlags |= Ci.nsIChannel.LOAD_CALL_CONTENT_SNIFFERS;
  if (channel instanceof Ci.nsIHttpChannel) {
    const referrer = Cc["@mozilla.org/referrer-info;1"].createInstance(Ci.nsIReferrerInfo);
    referrer.init(Ci.nsIReferrerInfo.EMPTY, true, browser.currentURI);
    channel.referrerInfo = referrer;
    if (channel instanceof Ci.nsIHttpChannelInternal) {
      channel.forceAllowThirdPartyCookie = true;
    }
    if (global.cookieJarSettings) {
      channel.loadInfo.cookieJarSettings = global.cookieJarSettings;
    }
  }
  return channel;
}

export function saveUri(window, browser, uri) {
  if (typeof uri !== "string" || !uri) {
    throw new Error("A download needs a URI");
  }
  const channel = channelFor(browser, uri);
  const helperApps = Cc["@mozilla.org/uriloader/external-helper-app-service;1"].getService(
    Ci.nsIExternalHelperAppService
  );
  return new Promise((resolve, reject) => {
    let saving = null;
    channel.asyncOpen({
      QueryInterface: ChromeUtils.generateQI(["nsIStreamListener", "nsIRequestObserver"]),
      onStartRequest(request) {
        if (!Components.isSuccessCode(request.status)) {
          reject(new Error(`The download failed with ${request.status}`));
          return;
        }
        try {
          saving = helperApps.doContent(channel.contentType, request, window.docShell, true, window);
          saving.onStartRequest(request);
        } catch (error) {
          reject(error);
          throw error;
        }
        resolve();
      },
      onDataAvailable(request, stream, offset, count) {
        saving?.onDataAvailable(request, stream, offset, count);
      },
      onStopRequest(request, status) {
        if (!saving) {
          reject(new Error(`The download failed with ${status}`));
          return;
        }
        saving.onStopRequest(request, status);
      },
    });
  });
}
