/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

const lazy = {};

ChromeUtils.defineESModuleGetters(lazy, {
  EventDispatcher: "resource://gre/modules/Messaging.sys.mjs",
  RikkaEditingParent: "resource://gre/modules/RikkaEditingParent.sys.mjs",
});

const EVENTS = ["Rikka:View:Edit", "Rikka:View:ReplaceMisspelling", "Rikka:View:AddToDictionary"];

function focusedFrame(browser) {
  const focused = Services.focus.focusedContentBrowsingContext;
  return focused?.top === browser.browsingContext ? focused : browser.browsingContext;
}

function actorOf(frame) {
  const actor = frame?.currentWindowGlobal?.getActor("RikkaEditing");
  if (!actor) {
    throw new Error("The frame has no document");
  }
  return actor;
}

export const RikkaEditing = {
  attach(window) {
    lazy.EventDispatcher.for(window).registerListener(
      {
        async onEvent(event, data, callback) {
          const browser = window.browser;
          if (!browser) {
            callback?.onError("The view has no browser yet");
            return;
          }
          try {
            switch (event) {
              case "Rikka:View:Edit":
                await actorOf(focusedFrame(browser)).sendQuery("Edit", { command: data?.command });
                break;
              case "Rikka:View:ReplaceMisspelling":
                await actorOf(lazy.RikkaEditingParent.menuFrame(browser)).sendQuery("ReplaceMisspelling", {
                  suggestion: String(data?.suggestion ?? ""),
                });
                break;
              case "Rikka:View:AddToDictionary":
                await actorOf(lazy.RikkaEditingParent.menuFrame(browser)).sendQuery("AddToDictionary");
                break;
            }
            callback?.onSuccess();
          } catch (error) {
            callback?.onError(String(error?.message ?? error));
          }
        },
      },
      EVENTS
    );
  },

  register() {
    ChromeUtils.registerWindowActor("RikkaEditing", {
      parent: {
        esModuleURI: "resource://gre/modules/RikkaEditingParent.sys.mjs",
      },
      child: {
        esModuleURI: "resource://gre/modules/RikkaEditingChild.sys.mjs",
        events: {
          contextmenu: { capture: true, mozSystemGroup: true },
        },
      },
      allFrames: true,
      messageManagerGroups: ["browsers"],
      safeForUntrustedWebProcess: true,
    });
  },
};
