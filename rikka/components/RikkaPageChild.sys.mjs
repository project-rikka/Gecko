/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

const lazy = {};

ChromeUtils.defineESModuleGetters(lazy, {
  setTimeout: "resource://gre/modules/Timer.sys.mjs",
  clearTimeout: "resource://gre/modules/Timer.sys.mjs",
});

const ICON_RELS = new Set(["icon", "apple-touch-icon", "apple-touch-icon-precomposed", "mask-icon"]);
const SETTLE_MS = 100;

export class RikkaPageChild extends JSWindowActorChild {
  #icons = new Map();
  #timer = null;

  handleEvent(event) {
    switch (event.type) {
      case "DOMLinkAdded":
      case "DOMLinkChanged":
        this.#link(event.target);
        break;
      case "DOMContentLoaded":
        if (event.target === this.document) {
          this.#report();
        }
        break;
    }
  }

  receiveMessage({ name }) {
    if (name !== "DocumentSize") {
      return undefined;
    }
    const root = this.document.documentElement;
    const body = this.document.body;
    return {
      width: Math.max(root?.scrollWidth ?? 0, body?.scrollWidth ?? 0),
      height: Math.max(root?.scrollHeight ?? 0, body?.scrollHeight ?? 0),
    };
  }

  didDestroy() {
    if (this.#timer) {
      lazy.clearTimeout(this.#timer);
    }
  }

  #link(link) {
    if (link.ownerDocument !== this.document || !link.href) {
      return;
    }
    const rels = (link.rel || "").toLowerCase().split(/\s+/).filter(Boolean);
    if (!rels.some(rel => ICON_RELS.has(rel))) {
      return;
    }
    this.#icons.set(link.href, {
      url: link.href,
      rel: rels.join(" "),
      sizes: link.sizes ? [...link.sizes] : [],
      type: link.type || null,
    });
    if (this.document.readyState !== "loading") {
      if (this.#timer) {
        lazy.clearTimeout(this.#timer);
      }
      this.#timer = lazy.setTimeout(() => {
        this.#timer = null;
        this.#report();
      }, SETTLE_MS);
    }
  }

  #report() {
    const icons = [...this.#icons.values()];
    const protocol = this.document.location?.protocol;
    if (!icons.length && (protocol === "http:" || protocol === "https:")) {
      icons.push({
        url: new URL("/favicon.ico", this.document.documentURI).href,
        rel: "icon",
        sizes: [],
        type: null,
      });
    }
    this.sendAsyncMessage("Icons", { page: this.document.documentURI, icons });
  }
}
