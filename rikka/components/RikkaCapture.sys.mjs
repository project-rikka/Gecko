/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

const lazy = {};

ChromeUtils.defineESModuleGetters(lazy, {
  EventDispatcher: "resource://gre/modules/Messaging.sys.mjs",
});

export const RikkaCapture = {
  attach(window) {
    lazy.EventDispatcher.for(window).registerListener(
      {
        onEvent(event, data, callback) {
          const browser = window.browser;
          if (!browser) {
            callback?.onError("The view has no browser yet");
            return;
          }
          const onlyScreen = !!data?.onlyScreen;
          for (const frame of browser.browsingContext.getAllBrowsingContextsInSubtree()) {
            try {
              frame.currentWindowGlobal?.getActor("RikkaCapture").sendAsyncMessage("Stop", { onlyScreen });
            } catch {}
          }
          callback?.onSuccess();
        },
      },
      ["Rikka:View:StopCapture"]
    );
  },

  register() {
    ChromeUtils.registerWindowActor("RikkaCapture", {
      parent: {
        esModuleURI: "resource://gre/modules/RikkaCaptureParent.sys.mjs",
      },
      child: {
        esModuleURI: "resource://gre/modules/RikkaCaptureChild.sys.mjs",
      },
      allFrames: true,
      messageManagerGroups: ["browsers"],
      safeForUntrustedWebProcess: true,
    });
    ChromeUtils.registerProcessActor("RikkaCaptureProcess", {
      child: {
        esModuleURI: "resource://gre/modules/RikkaCaptureProcessChild.sys.mjs",
        observers: ["recording-device-events"],
      },
      safeForUntrustedWebProcess: true,
    });
  },
};
