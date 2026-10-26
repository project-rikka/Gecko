/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

const lazy = {};

ChromeUtils.defineESModuleGetters(lazy, {
  Downloads: "resource://gre/modules/Downloads.sys.mjs",
  EventDispatcher: "resource://gre/modules/Messaging.sys.mjs",
});

const COMMANDS = [
  "Rikka:Download:Cancel",
  "Rikka:Download:Pause",
  "Rikka:Download:Resume",
  "Rikka:Download:Remove",
];

function stateOf(download) {
  if (download.succeeded) {
    return "finished";
  }
  if (download.error) {
    return "failed";
  }
  if (download.canceled) {
    return download.hasPartialData ? "paused" : "cancelled";
  }
  return "downloading";
}

export class RikkaDownloads {
  #ids = new Map();
  #downloads = new Map();
  #next = 1;
  #list = null;

  observe(subject, topic) {
    if (topic === "profile-after-change") {
      this.#start();
    }
  }

  async #start() {
    lazy.EventDispatcher.instance.registerListener(this, COMMANDS);
    this.#list = await lazy.Downloads.getList(lazy.Downloads.ALL);
    await this.#list.addView(this);
  }

  #id(download) {
    let id = this.#ids.get(download);
    if (!id) {
      id = String(this.#next++);
      this.#ids.set(download, id);
      this.#downloads.set(id, download);
    }
    return id;
  }

  #report(download) {
    lazy.EventDispatcher.instance.sendRequest("Rikka:Download:Changed", {
      id: this.#id(download),
      url: download.source.url,
      referrer: download.source.referrerInfo?.originalReferrer?.spec ?? null,
      private: download.source.isPrivate,
      path: download.target.path,
      mimeType: download.contentType ?? null,
      state: stateOf(download),
      currentBytes: download.currentBytes,
      totalBytes: download.hasProgress ? download.totalBytes : null,
      startTime: download.startTime ? download.startTime.getTime() : null,
      error: download.error ? String(download.error.message) : null,
    });
  }

  onDownloadAdded(download) {
    this.#report(download);
  }

  onDownloadChanged(download) {
    this.#report(download);
  }

  onDownloadRemoved(download) {
    const id = this.#ids.get(download);
    if (!id) {
      return;
    }
    this.#ids.delete(download);
    this.#downloads.delete(id);
    lazy.EventDispatcher.instance.sendRequest("Rikka:Download:Removed", { id });
  }

  async onEvent(event, data, callback) {
    const download = this.#downloads.get(String(data?.id));
    if (!download) {
      callback?.onError(`No download ${data?.id}`);
      return;
    }
    try {
      switch (event) {
        case "Rikka:Download:Cancel":
          download.tryToKeepPartialData = false;
          await download.cancel();
          await download.removePartialData();
          break;
        case "Rikka:Download:Pause":
          download.tryToKeepPartialData = true;
          await download.cancel();
          break;
        case "Rikka:Download:Resume":
          download.start().catch(() => {});
          break;
        case "Rikka:Download:Remove":
          await download.finalize(true);
          await this.#list.remove(download);
          break;
      }
      callback?.onSuccess();
    } catch (error) {
      callback?.onError(String(error));
    }
  }
}
