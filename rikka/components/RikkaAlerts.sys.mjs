/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

const lazy = {};

ChromeUtils.defineESModuleGetters(lazy, {
  EventDispatcher: "resource://gre/modules/Messaging.sys.mjs",
});

const ANSWERS = ["Rikka:Notification:Clicked", "Rikka:Notification:Closed"];

export class RikkaAlerts {
  static QueryInterface = ChromeUtils.generateQI(["nsIAlertsService"]);

  #shown = new Map();
  #listening = false;

  #listen() {
    if (!this.#listening) {
      lazy.EventDispatcher.instance.registerListener(this, ANSWERS);
      this.#listening = true;
    }
  }

  showAlert(alert, listener) {
    this.#listen();
    const name = alert.name || alert.id || String(Services.uuid.generateUUID());
    this.#close(name, false);
    this.#shown.set(name, { alert, listener });

    lazy.EventDispatcher.instance.sendRequest("Rikka:Notification:Show", {
      name,
      title: alert.title,
      text: alert.text,
      imageUrl: alert.imageURL || null,
      origin: alert.principal?.isContentPrincipal ? alert.principal.origin : null,
      lang: alert.lang || null,
      dir: alert.dir || null,
      silent: alert.silent,
      requireInteraction: alert.requireInteraction,
      private: alert.inPrivateBrowsing,
      actions: (alert.actions ?? []).map(action => ({
        action: action.action,
        title: action.title,
      })),
    });
    listener?.observe(null, "alertshow", alert.cookie);
  }

  showAlertWithCallbacks(alert, callbacks) {
    this.showAlert(alert, callbacks);
  }

  closeAlert(name) {
    this.#close(name, true);
  }

  #close(name, tell) {
    const shown = this.#shown.get(name);
    if (!shown) {
      return;
    }
    this.#shown.delete(name);
    if (tell) {
      lazy.EventDispatcher.instance.sendRequest("Rikka:Notification:Close", { name });
    }
    shown.listener?.observe(null, "alertfinished", shown.alert.cookie);
  }

  getHistory() {
    return [...this.#shown.keys()];
  }

  teardown() {
    for (const name of [...this.#shown.keys()]) {
      this.#close(name, true);
    }
  }

  pbmTeardown() {
    for (const [name, shown] of [...this.#shown]) {
      if (shown.alert.inPrivateBrowsing) {
        this.#close(name, true);
      }
    }
  }

  isFullscreen() {
    return false;
  }

  onEvent(event, data) {
    const shown = this.#shown.get(data?.name);
    if (!shown) {
      return;
    }
    if (event === "Rikka:Notification:Clicked") {
      const action = shown.alert.actions?.find(entry => entry.action === data.action) ?? null;
      shown.listener?.observe(action, "alertclickcallback", shown.alert.cookie);
    }
    this.#close(data.name, false);
  }
}
