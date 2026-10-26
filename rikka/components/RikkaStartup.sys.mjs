/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

const lazy = {};

ChromeUtils.defineESModuleGetters(lazy, {
  RikkaDownloads: "resource://gre/modules/RikkaDownloads.sys.mjs",
  RikkaUserContent: "resource://gre/modules/RikkaUserContent.sys.mjs",
  RikkaView: "resource://gre/modules/RikkaView.sys.mjs",
});

export class RikkaStartup {
  observe(subject, topic) {
    if (topic !== "profile-after-change") {
      return;
    }
    lazy.RikkaUserContent.register();
    lazy.RikkaView.register();
    new lazy.RikkaDownloads().start();
  }
}
