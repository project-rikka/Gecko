/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

const lazy = {};

ChromeUtils.defineESModuleGetters(lazy, {
  BrowserUtils: "resource://gre/modules/BrowserUtils.sys.mjs",
});

const ORIGIN_ONLY = new Set(["origin", "strict-origin"]);

function whereOf(event) {
  if (event.altKey || event.metaKey) {
    return null;
  }
  const tab = event.shiftKey
    ? Ci.nsIBrowserDOMWindow.OPEN_NEWTAB_FOREGROUND
    : Ci.nsIBrowserDOMWindow.OPEN_NEWTAB_BACKGROUND;
  if (event.button === 1 || (event.button === 0 && event.ctrlKey)) {
    return tab;
  }
  if (event.button === 0 && event.shiftKey) {
    return Ci.nsIBrowserDOMWindow.OPEN_NEWWINDOW;
  }
  return null;
}

function isEditable(target) {
  const className = ChromeUtils.getClassName(target);
  return (
    target.isContentEditable ||
    target.ownerDocument?.designMode === "on" ||
    className === "HTMLInputElement" ||
    className === "HTMLTextAreaElement"
  );
}

function referrerOf(node, document, href) {
  const info = Cc["@mozilla.org/referrer-info;1"].createInstance(Ci.nsIReferrerInfo);
  if (node) {
    info.initWithElement(node);
  } else {
    info.initWithDocument(document);
  }
  const original = info.originalReferrer;
  if (!info.sendReferrer || !original) {
    return null;
  }
  const policy = info.getReferrerPolicyString();
  if (policy === "no-referrer") {
    return null;
  }
  if (policy === "same-origin") {
    return URL.parse(href)?.origin === URL.parse(original.spec)?.origin ? original.specIgnoringRef : null;
  }
  if (ORIGIN_ONLY.has(policy)) {
    return `${original.prePath}/`;
  }
  return original.specIgnoringRef;
}

export class RikkaLinkChild extends JSWindowActorChild {
  handleEvent(wrapper) {
    const event = wrapper.sourceEvent;
    if (!event || event.defaultPrevented) {
      return;
    }
    const where = whereOf(event);
    if (where === null || isEditable(event.composedTarget)) {
      return;
    }
    const document = event.originalTarget?.ownerDocument;
    if (!document || (!event.isTrusted && !document.hasValidTransientUserGestureActivation)) {
      return;
    }
    const [href, node, principal] = lazy.BrowserUtils.hrefAndLinkNodeForClickEvent(event) ?? [];
    if (!href || Services.io.extractScheme(href) === "javascript") {
      return;
    }
    try {
      Services.scriptSecurityManager.checkLoadURIStrWithPrincipal(principal, href);
    } catch {
      return;
    }
    if (!event.isTrusted) {
      document.consumeTransientUserGestureActivation();
    }
    event.preventMultipleActions();
    event.preventDefault();
    wrapper.preventDefault();
    this.sendAsyncMessage("OpenLink", { uri: href, referrer: referrerOf(node, document, href), where });
  }
}
