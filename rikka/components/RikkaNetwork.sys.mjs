/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

const lazy = {};

ChromeUtils.defineESModuleGetters(lazy, {
  EventDispatcher: "resource://gre/modules/Messaging.sys.mjs",
});

const EVENTS = ["Rikka:Network:Preconnect"];

function preconnect(data) {
  const uri = Services.io.newURI(String(data?.uri ?? ""));
  if (!uri.schemeIs("http") && !uri.schemeIs("https")) {
    return;
  }
  const principal = Services.scriptSecurityManager.createContentPrincipal(uri, {
    privateBrowsingId: data?.private ? 1 : 0,
  });
  Services.io.speculativeConnect(uri, principal, null, false);
}

export class RikkaNetwork {
  start() {
    lazy.EventDispatcher.instance.registerListener(this, EVENTS);
  }

  onEvent(event, data) {
    if (event !== "Rikka:Network:Preconnect") {
      return;
    }
    try {
      preconnect(data);
    } catch (error) {
      console.debug(`no speculative connection: ${error}`);
    }
  }
}
