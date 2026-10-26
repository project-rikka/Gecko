/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

const lazy = {};

ChromeUtils.defineESModuleGetters(lazy, {
  EventDispatcher: "resource://gre/modules/Messaging.sys.mjs",
});

const EVENTS = ["Rikka:View:SetZoom", "Rikka:View:Snapshot", "Rikka:View:SavePdf"];

const HTML = "http://www.w3.org/1999/xhtml";

function encodedPng(window, bitmap) {
  const canvas = window.document.createElementNS(HTML, "canvas");
  canvas.width = bitmap.width;
  canvas.height = bitmap.height;
  canvas.getContext("2d").drawImage(bitmap, 0, 0);
  bitmap.close();
  return canvas.toDataURL("image/png").slice("data:image/png;base64,".length);
}

async function snapshot(window, browser, data) {
  const global = browser.browsingContext?.currentWindowGlobal;
  if (!global) {
    throw new Error("The page has no document yet");
  }
  const scale = typeof data?.scale === "number" && data.scale > 0 ? data.scale : window.devicePixelRatio;
  let rect = null;
  if (data?.region === "document") {
    const size = await global.getActor("RikkaPage").sendQuery("DocumentSize");
    rect = new window.DOMRect(0, 0, size.width, size.height);
  }
  const bitmap = await global.drawSnapshot(rect, scale, "white", { resetScrollPosition: rect !== null });
  return { width: bitmap.width, height: bitmap.height, png: encodedPng(window, bitmap) };
}

async function savePdf(browser, path) {
  if (typeof path !== "string" || !PathUtils.isAbsolute(path)) {
    throw new Error("A PDF needs an absolute path");
  }
  const service = Cc["@mozilla.org/gfx/printsettings-service;1"].getService(Ci.nsIPrintSettingsService);
  const settings = service.createNewPrintSettings();
  settings.printSilent = true;
  settings.outputDestination = Ci.nsIPrintSettings.kOutputDestinationFile;
  settings.outputFormat = Ci.nsIPrintSettings.kOutputFormatPDF;
  settings.toFileName = path;
  await browser.browsingContext.print(settings);
}

function browserWindow(dispatcher) {
  return {
    QueryInterface: ChromeUtils.generateQI(["nsIXULBrowserWindow"]),
    setOverLink(link) {
      dispatcher.sendRequest("Rikka:View:OverLink", { url: link || null });
    },
    showTooltip(x, y, text, direction) {
      dispatcher.sendRequest("Rikka:View:Tooltip", { x, y, text, direction: direction || "ltr" });
    },
    hideTooltip() {
      dispatcher.sendRequest("Rikka:View:TooltipHidden", {});
    },
  };
}

export const RikkaView = {
  attach(window) {
    const dispatcher = lazy.EventDispatcher.for(window);

    window.docShell.treeOwner
      .QueryInterface(Ci.nsIInterfaceRequestor)
      .getInterface(Ci.nsIAppWindow).XULBrowserWindow = browserWindow(dispatcher);

    dispatcher.registerListener(
      {
        async onEvent(event, data, callback) {
          const browser = window.browser;
          if (!browser) {
            callback?.onError("The view has no browser yet");
            return;
          }
          try {
            switch (event) {
              case "Rikka:View:SetZoom":
                if (typeof data?.zoom !== "number" || !(data.zoom > 0)) {
                  throw new Error("Zoom must be a positive number");
                }
                browser.browsingContext.fullZoom = data.zoom;
                callback?.onSuccess();
                return;
              case "Rikka:View:Snapshot":
                callback?.onSuccess(await snapshot(window, browser, data));
                return;
              case "Rikka:View:SavePdf":
                await savePdf(browser, data?.path);
                callback?.onSuccess();
                return;
            }
          } catch (error) {
            callback?.onError(String(error?.message ?? error));
          }
        },
      },
      EVENTS
    );
  },

  register() {
    ChromeUtils.registerWindowActor("RikkaPage", {
      parent: {
        esModuleURI: "resource://gre/modules/RikkaPageParent.sys.mjs",
      },
      child: {
        esModuleURI: "resource://gre/modules/RikkaPageChild.sys.mjs",
        events: {
          DOMLinkAdded: {},
          DOMLinkChanged: {},
          DOMContentLoaded: { mozSystemGroup: true },
        },
      },
      messageManagerGroups: ["browsers"],
      safeForUntrustedWebProcess: true,
    });
  },
};
