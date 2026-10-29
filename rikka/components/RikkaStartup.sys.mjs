/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

const lazy = {};

ChromeUtils.defineESModuleGetters(lazy, {
  RikkaCapture: "resource://gre/modules/RikkaCapture.sys.mjs",
  RikkaCertificates: "resource://gre/modules/RikkaCertificates.sys.mjs",
  RikkaDownloads: "resource://gre/modules/RikkaDownloads.sys.mjs",
  RikkaEditing: "resource://gre/modules/RikkaEditing.sys.mjs",
  RikkaExtensionIcons: "resource://gre/modules/RikkaExtensionIcons.sys.mjs",
  RikkaNetwork: "resource://gre/modules/RikkaNetwork.sys.mjs",
  RikkaStorage: "resource://gre/modules/RikkaStorage.sys.mjs",
  RikkaUserContent: "resource://gre/modules/RikkaUserContent.sys.mjs",
  RikkaView: "resource://gre/modules/RikkaView.sys.mjs",
});

function registerLinks() {
  ChromeUtils.registerWindowActor("RikkaLink", {
    parent: {
      esModuleURI: "resource://gre/modules/RikkaLinkParent.sys.mjs",
    },
    child: {
      esModuleURI: "resource://gre/modules/RikkaLinkChild.sys.mjs",
      events: {
        chromelinkclick: { capture: true, mozSystemGroup: true },
      },
    },
    allFrames: true,
    messageManagerGroups: ["browsers"],
    safeForUntrustedWebProcess: true,
  });
}

export class RikkaStartup {
  observe(subject, topic) {
    if (topic !== "profile-after-change") {
      return;
    }
    lazy.RikkaUserContent.register();
    lazy.RikkaView.register();
    lazy.RikkaEditing.register();
    lazy.RikkaCapture.register();
    lazy.RikkaCertificates.register();
    registerLinks();
    new lazy.RikkaDownloads().start();
    new lazy.RikkaStorage().start();
    new lazy.RikkaExtensionIcons().start();
    new lazy.RikkaNetwork().start();
  }
}
