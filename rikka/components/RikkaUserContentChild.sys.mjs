/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

const compiled = new Map();
const patternSets = new Map();

function patternSet(patterns) {
  const key = patterns.join("\n");
  let set = patternSets.get(key);
  if (!set) {
    set = new MatchPatternSet(patterns, { restrictSchemes: false });
    patternSets.set(key, set);
  }
  return set;
}

function compile(source) {
  let entry = compiled.get(source);
  if (!entry) {
    const url = `data:text/javascript;charset=utf-8,${encodeURIComponent(source)}`;
    entry = { promise: ChromeUtils.compileScript(url, { hasReturnValue: true, filename: "<rikka user script>" }) };
    entry.promise.then(script => {
      entry.script = script;
    });
    compiled.set(source, entry);
  }
  return entry;
}

function jsonOf(value) {
  const text = JSON.stringify(value);
  return text === undefined ? null : JSON.parse(text);
}

const IDENTIFIER = /^[A-Za-z_$][A-Za-z0-9_$]*$/;

export class RikkaUserContentChild extends JSWindowActorChild {
  #sandboxes = new Map();
  #exposed = new Set();
  #handlerRoots = new Map();

  get #content() {
    return Services.cpmm.sharedData.get(`rikka-user-content:${this.browsingContext.browserId}`) ?? null;
  }

  get #top() {
    return this.browsingContext === this.browsingContext.top;
  }

  observe(subject, topic) {
    if (topic === "document-element-inserted" && subject === this.document) {
      this.#inject("start");
    }
  }

  handleEvent(event) {
    if (event.type === "DOMContentLoaded" && event.target === this.document) {
      this.#inject("end");
    }
  }

  async receiveMessage({ name, data }) {
    switch (name) {
      case "Evaluate":
        return jsonOf(await this.#evaluate(data.world ?? null, data.source));
      case "Call":
        return jsonOf(await this.#call(data.world ?? null, data.body, data.arguments ?? {}));
    }
    return undefined;
  }

  #applies(entry) {
    if (entry.frames === "top" && !this.#top) {
      return false;
    }
    const uri = this.document.documentURIObject;
    if (entry.allow?.length && !patternSet(entry.allow).matches(uri)) {
      return false;
    }
    return !(entry.block?.length && patternSet(entry.block).matches(uri));
  }

  #inject(at) {
    const content = this.#content;
    const window = this.contentWindow;
    if (!content || !window) {
      return;
    }

    if (at === "start") {
      for (const handler of content.handlers) {
        this.#expose(handler);
      }
      for (const style of content.styles) {
        if (this.#applies(style)) {
          this.#style(style);
        }
      }
    }

    const scripts = content.scripts.filter(script => script.injectAt === at && this.#applies(script));
    const pending = scripts
      .filter(script => script.world === null)
      .map(script => compile(script.source))
      .filter(entry => !entry.script);
    if (pending.length && at === "start") {
      this.document.blockParsing(
        Promise.all(pending.map(entry => entry.promise)).then(() => this.#run(scripts)),
        { blockScriptCreated: true }
      );
      return;
    }
    this.#run(scripts);
  }

  #run(scripts) {
    for (const script of scripts) {
      try {
        if (script.world === null) {
          compile(script.source).script?.executeInGlobal(this.contentWindow, { reportExceptions: true });
        } else {
          Cu.evalInSandbox(script.source, this.#sandbox(script.world), "latest", `rikka:${script.world}:${script.id}`, 1);
        }
      } catch (error) {
        console.error(error);
      }
    }
  }

  #style(style) {
    const utils = this.contentWindow.windowUtils;
    const url = `data:text/css;charset=utf-8,${encodeURIComponent(style.source)}`;
    utils.loadSheetUsingURIString(url, style.level === "author" ? utils.AUTHOR_SHEET : utils.USER_SHEET);
  }

  #sandbox(world) {
    let sandbox = this.#sandboxes.get(world);
    if (!sandbox) {
      const window = this.contentWindow;
      sandbox = Cu.Sandbox(window, {
        sandboxName: `rikka world ${world}`,
        sandboxPrototype: window,
        sameZoneAs: window,
        wantXrays: true,
      });
      this.#sandboxes.set(world, sandbox);
    }
    return sandbox;
  }

  #scope(world) {
    return world === null ? this.contentWindow : this.#sandbox(world);
  }

  #expose(handler) {
    const world = handler.world ?? null;
    const key = `${world}\n${handler.name}`;
    if (this.#exposed.has(key)) {
      return;
    }
    this.#exposed.add(key);

    const scope = this.#scope(world);
    let handlers = this.#handlerRoots.get(world);
    if (!handlers) {
      const root = Cu.createObjectIn(scope, { defineAs: "rikka" });
      handlers = Cu.createObjectIn(root, { defineAs: "messageHandlers" });
      if (world !== null) {
        Cu.evalInSandbox("window.rikka = rikka;", scope, "latest", `rikka:${world}:handlers`, 1);
      }
      this.#handlerRoots.set(world, handlers);
    }
    const target = Cu.createObjectIn(handlers, { defineAs: handler.name });

    Cu.exportFunction(
      body => {
        const message = {
          name: handler.name,
          world,
          body: jsonOf(body),
          frame: { url: this.document.documentURI, top: this.#top },
          replying: handler.replying,
        };
        if (!handler.replying) {
          this.sendAsyncMessage("Message", message);
          return undefined;
        }
        return new scope.Promise((resolve, reject) => {
          this.sendQuery("Message", message).then(
            result => resolve(Cu.cloneInto(result, scope)),
            error => reject(new scope.Error(String(error?.message ?? error)))
          );
        });
      },
      target,
      { defineAs: "postMessage" }
    );
  }

  async #evaluate(world, source) {
    if (world === null) {
      const entry = compile(source);
      const script = entry.script ?? (await entry.promise);
      return Cu.waiveXrays(await script.executeInGlobal(this.contentWindow, { reportExceptions: false }));
    }
    return Cu.waiveXrays(await Cu.evalInSandbox(source, this.#sandbox(world), "latest", `rikka:${world}:evaluate`, 1));
  }

  async #call(world, body, args) {
    const names = Object.keys(args);
    if (!names.every(name => IDENTIFIER.test(name))) {
      throw new Error("Call arguments must be named by identifiers");
    }
    const source = `(async function (${names.join(", ")}) {\n${body}\n})`;
    const scope = this.#scope(world);
    const fn = world === null ? await this.#evaluate(null, source) : Cu.evalInSandbox(source, scope, "latest", `rikka:${world}:call`, 1);
    const values = names.map(name => Cu.cloneInto(args[name], scope));
    return Cu.waiveXrays(await fn(...values));
  }
}
