/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

const lazy = {};

ChromeUtils.defineESModuleGetters(lazy, {
  EventDispatcher: "resource://gre/modules/Messaging.sys.mjs",
});

const REASONS = {
  [Ci.nsIHelperAppLauncherDialog.REASON_CANTHANDLE]: "unhandled",
  [Ci.nsIHelperAppLauncherDialog.REASON_SERVERREQUEST]: "attachment",
  [Ci.nsIHelperAppLauncherDialog.REASON_TYPESNIFFED]: "sniffed",
};

function dispatcherFor(launcher) {
  const window = BrowsingContext.get(launcher.browsingContextId)?.topChromeWindow;
  if (!window || window.closed) {
    return null;
  }
  try {
    return lazy.EventDispatcher.for(window);
  } catch {
    return null;
  }
}

function fileAt(path) {
  if (typeof path !== "string" || !PathUtils.isAbsolute(path)) {
    return null;
  }
  const file = Cc["@mozilla.org/file/local;1"].createInstance(Ci.nsIFile);
  file.initWithPath(path);
  return file;
}

async function askForDestination(launcher, reason, suggested) {
  const dispatcher = dispatcherFor(launcher);
  if (!dispatcher) {
    return null;
  }
  try {
    const answer = await dispatcher.sendRequestForResult("Rikka:DownloadRequest", {
      url: launcher.source.spec,
      filename: suggested || launcher.suggestedFileName,
      mimeType: launcher.MIMEInfo?.MIMEType ?? null,
      contentLength: launcher.contentLength >= 0 ? launcher.contentLength : null,
      reason,
    });
    return fileAt(answer?.path);
  } catch {
    return null;
  }
}

export class RikkaHelperAppDialog {
  static QueryInterface = ChromeUtils.generateQI(["nsIHelperAppLauncherDialog"]);

  #chosen = new WeakMap();

  show(launcher, context, reason) {
    askForDestination(launcher, REASONS[reason] ?? "unhandled", null).then(file => {
      if (!file) {
        launcher.cancel(Cr.NS_BINDING_ABORTED);
        return;
      }
      this.#chosen.set(launcher, file);
      launcher.promptForSaveDestination();
    });
  }

  promptForSaveToFileAsync(launcher, context, defaultFile, suggestedExtension) {
    const chosen = this.#chosen.get(launcher);
    if (chosen) {
      this.#chosen.delete(launcher);
      launcher.saveDestinationAvailable(chosen, true);
      return;
    }
    const suggested = defaultFile ? defaultFile + (suggestedExtension && !defaultFile.endsWith(suggestedExtension) ? suggestedExtension : "") : null;
    askForDestination(launcher, "save", suggested).then(file => {
      launcher.saveDestinationAvailable(file, true);
    });
  }
}
