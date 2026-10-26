/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

const lazy = {};

ChromeUtils.defineESModuleGetters(lazy, {
  EventDispatcher: "resource://gre/modules/Messaging.sys.mjs",
  NetUtil: "resource://gre/modules/NetUtil.sys.mjs",
});

const MAX_ICON_BYTES = 4 * 1024 * 1024;

function fetchIcon(spec) {
  const uri = Services.io.newURI(String(spec ?? ""));
  if (!uri.schemeIs("moz-extension")) {
    throw new Error("Extension icons are moz-extension URIs");
  }
  if (!WebExtensionPolicy.getByURI(uri)) {
    throw new Error("No extension serves this icon");
  }
  const channel = lazy.NetUtil.newChannel({ uri, loadUsingSystemPrincipal: true });
  return new Promise((resolve, reject) => {
    lazy.NetUtil.asyncFetch(channel, (stream, status) => {
      if (!Components.isSuccessCode(status)) {
        reject(new Error(`The icon could not be read: ${status}`));
        return;
      }
      try {
        const available = stream.available();
        if (available > MAX_ICON_BYTES) {
          throw new Error("The icon is too large");
        }
        const bytes = lazy.NetUtil.readInputStreamToString(stream, available);
        resolve({ mimeType: channel.contentType || null, data: btoa(bytes) });
      } catch (error) {
        reject(error);
      } finally {
        stream.close();
      }
    });
  });
}

export class RikkaExtensionIcons {
  start() {
    lazy.EventDispatcher.instance.registerListener(this, ["Rikka:Extension:Icon"]);
  }

  async onEvent(event, data, callback) {
    try {
      callback?.onSuccess(await fetchIcon(data?.uri));
    } catch (error) {
      callback?.onError(String(error?.message ?? error));
    }
  }
}
