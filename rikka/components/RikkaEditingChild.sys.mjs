/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

const lazy = {};

ChromeUtils.defineESModuleGetters(lazy, {
  InlineSpellChecker: "resource://gre/modules/InlineSpellChecker.sys.mjs",
  SelectionUtils: "resource://gre/modules/SelectionUtils.sys.mjs",
  SpellCheckHelper: "resource://gre/modules/InlineSpellChecker.sys.mjs",
});

const MAX_TEXT_LENGTH = 4096;
const SUGGESTIONS = 5;

const COMMANDS = {
  cut: "cmd_cut",
  copy: "cmd_copy",
  paste: "cmd_paste",
  pastePlainText: "cmd_pasteNoFormatting",
  selectAll: "cmd_selectAll",
  undo: "cmd_undo",
  redo: "cmd_redo",
  delete: "cmd_delete",
};

const MEDIA_TYPES = new Set(["HTMLImageElement", "HTMLVideoElement", "HTMLAudioElement"]);

function clipped(text) {
  return (text && text.substring(0, MAX_TEXT_LENGTH)) || null;
}

function nearestAttribute(node, attribute) {
  while (node && node.hasAttribute && !node.hasAttribute(attribute)) {
    node = node.parentNode;
  }
  return node?.getAttribute?.(attribute) ?? null;
}

function absolute(base, uri) {
  if (!uri || !base?.displaySpec) {
    return null;
  }
  try {
    return Services.io.newURI(uri, null, base).displaySpec;
  } catch {
    return null;
  }
}

function sourceOf(node, className) {
  if (!MEDIA_TYPES.has(className)) {
    return null;
  }
  const source = node.currentSrc || node.src;
  if (!source || (source.startsWith("blob:") && !URL.isBoundToBlob(source))) {
    return null;
  }
  return source;
}

function selectedText(window) {
  try {
    return lazy.SelectionUtils.getSelectionDetails(window, MAX_TEXT_LENGTH).fullText || null;
  } catch {
    return null;
  }
}

function editorOf(event, flags) {
  if (flags & (lazy.SpellCheckHelper.TEXTAREA | lazy.SpellCheckHelper.INPUT)) {
    return event.composedTarget.editor;
  }
  const window = event.target.documentGlobal;
  return window.docShell.editingSession?.getEditorForWindow(window) ?? null;
}

export class RikkaEditingChild extends JSWindowActorChild {
  #spelling = null;

  handleEvent(event) {
    if (event.type !== "contextmenu" || event.defaultPrevented) {
      return;
    }
    event.preventDefault();
    this.#describe(event).then(
      menu => this.sendAsyncMessage("ContextMenu", menu),
      error => console.error(error)
    );
  }

  async #describe(event) {
    const node = event.composedTarget;
    const window = this.contentWindow;
    const base = node.ownerDocument.baseURIObject;
    const className = ChromeUtils.getClassName(node);
    const flags = lazy.SpellCheckHelper.isEditable(node, window);
    const editable = !!(flags & (lazy.SpellCheckHelper.EDITABLE | lazy.SpellCheckHelper.CONTENTEDITABLE));
    const selection = selectedText(window);
    return {
      screenX: event.screenX,
      screenY: event.screenY,
      baseUri: base?.displaySpec || null,
      uri: absolute(base, nearestAttribute(node, "href")),
      title: clipped(nearestAttribute(node, "title")),
      alt: clipped(nearestAttribute(node, "alt")),
      elementType: MEDIA_TYPES.has(className) ? className : "none",
      elementSrc: sourceOf(node, className),
      textContent: clipped(node.textContent),
      linkText: clipped(node.innerText),
      selection,
      editable,
      spelling: editable ? await this.#spellingAt(event, flags) : null,
    };
  }

  async #spellingAt(event, flags) {
    this.#spelling = null;
    const editor = editorOf(event, flags);
    if (!editor) {
      return null;
    }
    const checker = new lazy.InlineSpellChecker(editor);
    checker.initFromEvent(event.rangeParent, event.rangeOffset);
    if (!checker.canSpellCheck || !checker.overMisspelling) {
      return null;
    }
    this.#spelling = checker;
    let suggestions = [];
    try {
      suggestions = await checker.mInlineSpellChecker.spellChecker.suggest(checker.mMisspelling, SUGGESTIONS);
    } catch {}
    return { misspelling: checker.mMisspelling, suggestions };
  }

  receiveMessage({ name, data }) {
    switch (name) {
      case "Edit": {
        const command = COMMANDS[data?.command];
        if (!command) {
          throw new Error(`Unknown edit command ${data?.command}`);
        }
        if (this.docShell.isCommandEnabled(command)) {
          this.docShell.doCommand(command);
        }
        return undefined;
      }
      case "ReplaceMisspelling":
        this.#spelling?.replaceMisspelling(String(data?.suggestion ?? ""));
        this.#spelling = null;
        return undefined;
      case "AddToDictionary":
        this.#spelling?.addToDictionary();
        this.#spelling = null;
        return undefined;
    }
    return undefined;
  }

  didDestroy() {
    this.#spelling?.uninit();
    this.#spelling = null;
  }
}
