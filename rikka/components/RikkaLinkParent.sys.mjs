/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

const lazy = {};

ChromeUtils.defineESModuleGetters(lazy, {
  EventDispatcher: "resource://gre/modules/Messaging.sys.mjs",
});

const TARGETS = new Set([
  Ci.nsIBrowserDOMWindow.OPEN_NEWWINDOW,
  Ci.nsIBrowserDOMWindow.OPEN_NEWTAB_BACKGROUND,
  Ci.nsIBrowserDOMWindow.OPEN_NEWTAB_FOREGROUND,
]);

export class RikkaLinkParent extends JSWindowActorParent {
  receiveMessage({ name, data }) {
    if (name !== "OpenLink" || !TARGETS.has(data?.where) || typeof data.uri !== "string") {
      return;
    }
    const window = this.browsingContext.topChromeWindow;
    if (!window || window.closed) {
      return;
    }
    lazy.EventDispatcher.for(window).sendRequest("Rikka:View:OpenLink", {
      uri: String(data.uri),
      referrer: typeof data.referrer === "string" ? data.referrer : null,
      where: data.where,
    });
  }
}
