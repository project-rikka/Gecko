/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

const lazy = {};

ChromeUtils.defineESModuleGetters(lazy, {
  EventDispatcher: "resource://gre/modules/Messaging.sys.mjs",
});

const EVENTS = ["Rikka:Storage:ClearBetween", "Rikka:Storage:SiteData", "Rikka:Storage:Cookies"];

const CLEAR_FLAGS = [
  [
    1 << 0,
    Ci.nsIClearDataService.CLEAR_COOKIES |
      Ci.nsIClearDataService.CLEAR_MEDIA_DEVICES |
      Ci.nsIClearDataService.CLEAR_FINGERPRINTING_PROTECTION_STATE |
      Ci.nsIClearDataService.CLEAR_BOUNCE_TRACKING_PROTECTION_STATE,
  ],
  [1 << 1, Ci.nsIClearDataService.CLEAR_NETWORK_CACHE],
  [1 << 2, Ci.nsIClearDataService.CLEAR_IMAGE_CACHE],
  [1 << 3, Ci.nsIClearDataService.CLEAR_HISTORY],
  [
    1 << 4,
    Ci.nsIClearDataService.CLEAR_DOM_QUOTA |
      Ci.nsIClearDataService.CLEAR_DOM_PUSH_NOTIFICATIONS |
      Ci.nsIClearDataService.CLEAR_REPORTS |
      Ci.nsIClearDataService.CLEAR_FINGERPRINTING_PROTECTION_STATE |
      Ci.nsIClearDataService.CLEAR_BOUNCE_TRACKING_PROTECTION_STATE,
  ],
  [1 << 5, Ci.nsIClearDataService.CLEAR_AUTH_TOKENS | Ci.nsIClearDataService.CLEAR_AUTH_CACHE],
  [1 << 6, Ci.nsIClearDataService.CLEAR_PERMISSIONS],
  [
    1 << 7,
    Ci.nsIClearDataService.CLEAR_CONTENT_PREFERENCES |
      Ci.nsIClearDataService.CLEAR_DOM_PUSH_NOTIFICATIONS |
      Ci.nsIClearDataService.CLEAR_CLIENT_AUTH_REMEMBER_SERVICE |
      Ci.nsIClearDataService.CLEAR_FINGERPRINTING_PROTECTION_STATE,
  ],
  [1 << 8, Ci.nsIClearDataService.CLEAR_EME | Ci.nsIClearDataService.CLEAR_HSTS],
  [1 << 9, Ci.nsIClearDataService.CLEAR_ALL],
];

const SAME_SITE = {
  [Ci.nsICookie.SAMESITE_NONE]: "none",
  [Ci.nsICookie.SAMESITE_LAX]: "lax",
  [Ci.nsICookie.SAMESITE_STRICT]: "strict",
};

function clearFlags(flags) {
  return CLEAR_FLAGS.filter(([bit]) => bit & flags).reduce((all, [, clear]) => all | clear, 0);
}

function attributesOf(data) {
  return {
    privateBrowsingId: data?.private ? 1 : 0,
    geckoViewSessionContextId: typeof data?.contextId === "string" ? data.contextId : "",
  };
}

function cookiesOf(host, attributes) {
  return Services.cookies
    .getCookiesWithOriginAttributes(JSON.stringify({ ...attributes, partitionKey: "" }), host)
    .filter(cookie => {
      const domain = cookie.rawHost;
      return cookie.isDomain ? host === domain || host.endsWith(`.${domain}`) : host === domain;
    });
}

function pathMatches(path, cookiePath) {
  return (
    path === cookiePath ||
    (path.startsWith(cookiePath) && (cookiePath.endsWith("/") || path[cookiePath.length] === "/"))
  );
}

function cookiesFor(data) {
  const uri = Services.io.newURI(String(data?.uri ?? ""));
  if (!uri.schemeIs("http") && !uri.schemeIs("https")) {
    throw new Error("Cookies belong to http and https URIs");
  }
  const secure = uri.schemeIs("https");
  return cookiesOf(uri.asciiHost, attributesOf(data))
    .filter(cookie => (secure || !cookie.isSecure) && pathMatches(uri.filePath, cookie.path))
    .map(cookie => ({
      name: cookie.name,
      value: cookie.value,
      host: cookie.host,
      path: cookie.path,
      secure: cookie.isSecure,
      httpOnly: cookie.isHttpOnly,
      expiry: cookie.isSession ? null : cookie.expiry,
      sameSite: SAME_SITE[cookie.sameSite] ?? "unset",
    }));
}

function usageOf(principal) {
  return new Promise(resolve => {
    Services.qms.getUsageForPrincipal(principal, {
      onUsageResult(request) {
        resolve(request.resultCode === Cr.NS_OK ? request.result.usage : 0);
      },
    });
  });
}

async function siteData(data) {
  const host = String(data?.host ?? "");
  if (!host) {
    throw new Error("Site data belongs to a host");
  }
  const attributes = attributesOf(data);
  const usages = await Promise.all(
    ["https", "http"].map(scheme =>
      usageOf(Services.scriptSecurityManager.createContentPrincipal(Services.io.newURI(`${scheme}://${host}`), attributes))
    )
  );
  return {
    cookies: cookiesOf(host, attributes).length,
    usage: usages.reduce((sum, usage) => sum + usage, 0),
  };
}

function clearBetween(data) {
  const from = data?.from;
  const to = data?.to;
  if (!Number.isSafeInteger(from) || !Number.isSafeInteger(to) || from > to) {
    throw new Error("A time range needs a start before its end");
  }
  return new Promise(resolve => {
    Services.clearData.deleteDataInTimeRange(from * 1000, to * 1000, true, clearFlags(data.flags), resolve);
  });
}

export class RikkaStorage {
  start() {
    lazy.EventDispatcher.instance.registerListener(this, EVENTS);
  }

  async onEvent(event, data, callback) {
    try {
      switch (event) {
        case "Rikka:Storage:ClearBetween":
          await clearBetween(data);
          callback?.onSuccess();
          return;
        case "Rikka:Storage:SiteData":
          callback?.onSuccess(await siteData(data));
          return;
        case "Rikka:Storage:Cookies":
          callback?.onSuccess(cookiesFor(data));
          return;
      }
    } catch (error) {
      callback?.onError(String(error?.message ?? error));
    }
  }
}
