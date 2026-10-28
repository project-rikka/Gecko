/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

const lazy = {};

ChromeUtils.defineESModuleGetters(lazy, {
  EventDispatcher: "resource://gre/modules/Messaging.sys.mjs",
});

ChromeUtils.defineLazyGetter(lazy, "errors", () =>
  Cc["@mozilla.org/nss_errors_service;1"].getService(Ci.nsINSSErrorsService)
);

const DEFAULT_PORT = 443;

function viewOf(loadInfo) {
  const frame = loadInfo.targetBrowsingContext ?? loadInfo.browsingContext;
  if (!frame || frame.parent) {
    return null;
  }
  const window = frame.topChromeWindow;
  if (!window || window.closed || window.document.documentElement.getAttribute("windowtype") !== "navigator:geckoview") {
    return null;
  }
  return window;
}

function failedCertificateOf(channel) {
  if (Components.isSuccessCode(channel.status) || !lazy.errors.isNSSErrorCode(channel.status)) {
    return null;
  }
  if (channel.loadInfo.externalContentPolicyType !== Ci.nsIContentPolicy.TYPE_DOCUMENT) {
    return null;
  }
  const security = channel.securityInfo;
  const certificate = security?.serverCert;
  if (!certificate) {
    return null;
  }
  const uri = channel.URI;
  return {
    uri: uri.spec,
    host: uri.asciiHost,
    port: uri.port === -1 ? DEFAULT_PORT : uri.port,
    certificate: certificate.getBase64DERString(),
    overridable: security.overridableErrorCategory !== Ci.nsITransportSecurityInfo.ERROR_UNSET,
  };
}

function allow(browser, data) {
  const host = typeof data?.host === "string" ? data.host : "";
  const port = data?.port;
  if (!host || !Number.isInteger(port) || port < 1 || port > 65535 || typeof data?.certificate !== "string") {
    throw new Error("A certificate exception needs a host, a port and a certificate");
  }
  const certificate = Cc["@mozilla.org/security/x509certdb;1"]
    .getService(Ci.nsIX509CertDB)
    .constructX509FromBase64(data.certificate);
  Cc["@mozilla.org/security/certoverride;1"]
    .getService(Ci.nsICertOverrideService)
    .rememberValidityOverride(host, port, browser.browsingContext.originAttributes, certificate, !data.permanent);
}

export const RikkaCertificates = {
  observe(subject) {
    let channel;
    try {
      channel = subject.QueryInterface(Ci.nsIHttpChannel);
    } catch {
      return;
    }
    const failure = failedCertificateOf(channel);
    const window = failure && viewOf(channel.loadInfo);
    if (window) {
      lazy.EventDispatcher.for(window).sendRequest("Rikka:View:CertificateError", failure);
    }
  },

  attach(window) {
    lazy.EventDispatcher.for(window).registerListener(
      {
        onEvent(event, data, callback) {
          const browser = window.browser;
          if (!browser) {
            callback?.onError("The view has no browser yet");
            return;
          }
          try {
            allow(browser, data);
            callback?.onSuccess();
          } catch (error) {
            callback?.onError(String(error?.message ?? error));
          }
        },
      },
      ["Rikka:View:AllowCertificate"]
    );
  },

  register() {
    Services.obs.addObserver(this, "http-on-before-stop-request");
  },
};
