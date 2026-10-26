/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

const lazy = {};

ChromeUtils.defineESModuleGetters(lazy, {
  EventDispatcher: "resource://gre/modules/Messaging.sys.mjs",
});

const EVENTS = ["Rikka:UserContent:Set", "Rikka:UserContent:Evaluate", "Rikka:UserContent:Call"];

function keyOf(browser) {
  return `rikka-user-content:${browser.browserId}`;
}

function topActor(browser) {
  const actor = browser.browsingContext?.currentWindowGlobal?.getActor("RikkaUserContent");
  if (!actor) {
    throw new Error("The page has no document yet");
  }
  return actor;
}

function stored(data) {
  return {
    scripts: (data?.scripts ?? []).map(script => ({
      id: String(script.id),
      source: String(script.source),
      injectAt: script.injectAt === "end" ? "end" : "start",
      frames: script.frames === "top" ? "top" : "all",
      world: typeof script.world === "string" ? script.world : null,
      allow: script.allow ?? [],
      block: script.block ?? [],
    })),
    styles: (data?.styles ?? []).map(style => ({
      id: String(style.id),
      source: String(style.source),
      frames: style.frames === "top" ? "top" : "all",
      level: style.level === "author" ? "author" : "user",
      allow: style.allow ?? [],
      block: style.block ?? [],
    })),
    handlers: (data?.handlers ?? []).map(handler => ({
      name: String(handler.name),
      world: typeof handler.world === "string" ? handler.world : null,
      replying: !!handler.replying,
    })),
  };
}

export const RikkaUserContent = {
  attach(window) {
    const dispatcher = lazy.EventDispatcher.for(window);
    const listener = {
      async onEvent(event, data, callback) {
        const browser = window.browser;
        if (!browser) {
          callback?.onError("The view has no browser yet");
          return;
        }
        try {
          switch (event) {
            case "Rikka:UserContent:Set":
              Services.ppmm.sharedData.set(keyOf(browser), stored(data));
              Services.ppmm.sharedData.flush();
              callback?.onSuccess();
              return;
            case "Rikka:UserContent:Evaluate":
              callback?.onSuccess(
                await topActor(browser).sendQuery("Evaluate", { world: data?.world ?? null, source: String(data?.source ?? "") })
              );
              return;
            case "Rikka:UserContent:Call":
              callback?.onSuccess(
                await topActor(browser).sendQuery("Call", {
                  world: data?.world ?? null,
                  body: String(data?.body ?? ""),
                  arguments: data?.arguments ?? {},
                })
              );
              return;
          }
        } catch (error) {
          callback?.onError(String(error?.message ?? error));
        }
      },
    };
    dispatcher.registerListener(listener, EVENTS);

    window.addEventListener(
      "unload",
      () => {
        if (window.browser) {
          Services.ppmm.sharedData.delete(keyOf(window.browser));
          Services.ppmm.sharedData.flush();
        }
      },
      { once: true }
    );
  },

  register() {
    ChromeUtils.registerWindowActor("RikkaUserContent", {
      parent: {
        esModuleURI: "resource://gre/modules/RikkaUserContentParent.sys.mjs",
      },
      child: {
        esModuleURI: "resource://gre/modules/RikkaUserContentChild.sys.mjs",
        events: {
          DOMContentLoaded: { mozSystemGroup: true },
        },
        observers: ["document-element-inserted"],
      },
      allFrames: true,
      messageManagerGroups: ["browsers"],
    });
  },
};
