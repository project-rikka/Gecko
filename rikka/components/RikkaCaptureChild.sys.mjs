/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

const lazy = {};

ChromeUtils.defineLazyGetter(lazy, "media", () =>
  Cc["@mozilla.org/mediaManagerService;1"].getService(Ci.nsIMediaManagerService)
);

function stateOf(value) {
  switch (value) {
    case Ci.nsIMediaManagerService.STATE_CAPTURE_ENABLED:
      return "live";
    case Ci.nsIMediaManagerService.STATE_CAPTURE_DISABLED:
      return "muted";
    default:
      return "off";
  }
}

function sharing(...states) {
  const values = states.map(stateOf);
  if (values.includes("live")) {
    return "live";
  }
  return values.includes("muted") ? "muted" : "off";
}

export class RikkaCaptureChild extends JSWindowActorChild {
  report() {
    const camera = {};
    const microphone = {};
    const screen = {};
    const window = {};
    const browser = {};
    const devices = {};
    lazy.media.mediaCaptureWindowState(this.contentWindow, camera, microphone, screen, window, browser, devices);
    this.sendAsyncMessage("Capture", {
      camera: stateOf(camera.value),
      microphone: stateOf(microphone.value),
      screen: sharing(screen.value, window.value, browser.value),
    });
  }

  receiveMessage({ name, data }) {
    if (name !== "Stop") {
      return;
    }
    const id = this.contentWindow?.windowGlobalChild?.innerWindowId;
    if (id === undefined) {
      return;
    }
    Services.obs.notifyObservers(null, "getUserMedia:revoke", data?.onlyScreen ? `screen:${id}` : String(id));
  }
}
