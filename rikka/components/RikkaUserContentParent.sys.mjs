/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

const lazy = {};

ChromeUtils.defineESModuleGetters(lazy, {
  EventDispatcher: "resource://gre/modules/Messaging.sys.mjs",
});

export class RikkaUserContentParent extends JSWindowActorParent {
  receiveMessage({ name, data }) {
    if (name !== "Message") {
      return undefined;
    }
    const window = this.browsingContext.top.embedderElement?.ownerGlobal;
    if (!window || window.closed) {
      return Promise.reject(new Error("The view is gone"));
    }
    const dispatcher = lazy.EventDispatcher.for(window);
    const message = {
      name: data.name,
      world: data.world,
      body: data.body,
      frame: data.frame,
    };
    if (!data.replying) {
      dispatcher.sendRequest("Rikka:UserContent:Message", message);
      return undefined;
    }
    return dispatcher.sendRequestForResult("Rikka:UserContent:Message", message);
  }
}
