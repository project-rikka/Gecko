/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

const lazy = {};

ChromeUtils.defineESModuleGetters(lazy, {
  EventDispatcher: "resource://gre/modules/Messaging.sys.mjs",
});

const STATES = new Set(["off", "live", "muted"]);

function checked(state) {
  return STATES.has(state) ? state : "off";
}

export class RikkaCaptureParent extends JSWindowActorParent {
  receiveMessage({ name, data }) {
    if (name !== "Capture") {
      return;
    }
    const window = this.browsingContext.topChromeWindow;
    if (!window || window.closed) {
      return;
    }
    lazy.EventDispatcher.for(window).sendRequest("Rikka:View:Capture", {
      camera: checked(data?.camera),
      microphone: checked(data?.microphone),
      screen: checked(data?.screen),
    });
  }
}
