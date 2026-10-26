/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

const lazy = {};

ChromeUtils.defineESModuleGetters(lazy, {
  EventDispatcher: "resource://gre/modules/Messaging.sys.mjs",
});

const menus = new WeakMap();

export class RikkaEditingParent extends JSWindowActorParent {
  static menuFrame(browser) {
    const frame = menus.get(browser);
    return frame && !frame.isDiscarded ? frame : null;
  }

  receiveMessage({ name, data }) {
    if (name !== "ContextMenu") {
      return;
    }
    const browser = this.browsingContext.top.embedderElement;
    const window = this.browsingContext.topChromeWindow;
    if (!browser || !window || window.closed) {
      return;
    }
    menus.set(browser, this.browsingContext);
    lazy.EventDispatcher.for(window).sendRequest("Rikka:View:ContextMenu", data);
  }
}
